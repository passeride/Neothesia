//! Deterministic attack scoring. Times are signed, real-time microseconds
//! relative to the song start; negative values accommodate the lead-in.
//! Feed timestamp-ordered attacks before advancing the input watermark.
use std::collections::VecDeque;

pub const WINDOW_US: i64 = 150_000;
pub const RULES_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub at: i64,
    pub pitch: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grade {
    Perfect,
    Great,
    Good,
    Miss,
    Extra,
}
impl Grade {
    pub fn label(self) -> &'static str {
        match self {
            Self::Perfect => "Perfect",
            Self::Great => "Great",
            Self::Good => "Good",
            Self::Miss => "Miss",
            Self::Extra => "Extra",
        }
    }
    fn points(self) -> u64 {
        match self {
            Self::Perfect => 100,
            Self::Great => 80,
            Self::Good => 50,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Feedback {
    pub pitch: u8,
    pub grade: Grade,
    pub error: i64,
}

#[derive(Clone, Debug)]
struct Group {
    range: std::ops::Range<usize>,
    deadline: i64,
}

#[derive(Clone, Debug)]
pub struct Chart {
    targets: Vec<Target>,
    groups: Vec<Group>,
    by_pitch: [Vec<usize>; 128],
    maximum: i128,
}
impl Chart {
    pub fn new(mut targets: Vec<Target>) -> Result<Self, &'static str> {
        if targets.is_empty() {
            return Err("Choose at least one pitched Play Along track.");
        }
        if targets.len() > 2_000_000 {
            return Err("This chart has too many targets.");
        }
        if targets
            .iter()
            .any(|t| t.pitch > 127 || t.at < 0 || t.at > i64::MAX - WINDOW_US)
        {
            return Err("Invalid target pitch or timestamp.");
        }
        targets.sort_by_key(|t| (t.at, t.pitch));
        targets.dedup();
        let mut groups = Vec::new();
        let mut start = 0;
        let mut maximum = 0;
        let mut by_pitch: [Vec<usize>; 128] = std::array::from_fn(|_| Vec::new());
        for (i, target) in targets.iter().enumerate() {
            by_pitch[target.pitch as usize].push(i);
            if i + 1 == targets.len() || targets[i + 1].at != target.at {
                maximum += (i + 1 - start) as i128 * 100 * multiplier(groups.len() as u64) as i128;
                groups.push(Group {
                    range: start..i + 1,
                    deadline: target.at + WINDOW_US,
                });
                start = i + 1;
            }
        }
        Ok(Self {
            targets,
            groups,
            by_pitch,
            maximum,
        })
    }
    pub fn target_count(&self) -> usize {
        self.targets.len()
    }
    pub fn end(&self) -> i64 {
        self.groups.last().unwrap().deadline
    }
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }
}

pub fn multiplier(streak: u64) -> u8 {
    match streak {
        0..=9 => 1,
        10..=24 => 2,
        25..=49 => 3,
        _ => 4,
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub perfect: u64,
    pub great: u64,
    pub good: u64,
    pub misses: u64,
    pub extras: u64,
    pub streak: u64,
    pub longest_streak: u64,
    pub early: u64,
    pub late: u64,
    pub timing_sum_us: i128,
    /// Fifteen 20 ms buckets, from [-150,-130) through [130,150].
    pub histogram: [u64; 15],
}
impl Stats {
    pub fn hits(&self) -> u64 {
        self.perfect + self.great + self.good
    }
    pub fn quality(&self) -> u64 {
        self.perfect * 100 + self.great * 80 + self.good * 50
    }
    pub fn accuracy(&self, targets: usize) -> f64 {
        self.quality() as f64 / (targets as u64 + self.extras).max(1) as f64
    }
}

#[derive(Clone, Debug)]
pub struct Judge {
    chart: Chart,
    grades: Vec<Option<Grade>>,
    next_group: usize,
    raw: i128,
    watermark: i64,
    pub stats: Stats,
    feedback: VecDeque<Feedback>,
}
impl Judge {
    pub fn new(chart: Chart) -> Self {
        let grades = vec![None; chart.target_count()];
        Self {
            chart,
            grades,
            next_group: 0,
            raw: 0,
            watermark: i64::MIN,
            stats: Stats::default(),
            feedback: VecDeque::new(),
        }
    }
    pub fn chart(&self) -> &Chart {
        &self.chart
    }
    pub fn finished(&self) -> bool {
        self.next_group == self.chart.groups.len()
    }
    pub fn score(&self) -> u32 {
        (self.raw.clamp(0, self.chart.maximum) * 1_000_000 / self.chart.maximum) as u32
    }
    pub fn full_combo(&self) -> bool {
        self.finished() && self.stats.misses == 0 && self.stats.extras == 0
    }
    pub fn perfect_run(&self) -> bool {
        self.full_combo() && self.stats.perfect as usize == self.chart.target_count()
    }
    pub fn take_feedback(&mut self) -> impl Iterator<Item = Feedback> + '_ {
        self.feedback.drain(..)
    }
    fn emit(&mut self, feedback: Feedback) {
        if self.feedback.len() == 128 {
            self.feedback.pop_front();
        }
        self.feedback.push_back(feedback);
    }
    /// Returns false for stale/out-of-order input; callers must invalidate a
    /// record instead of quietly judging a queued attack at the current frame.
    pub fn attack(&mut self, at: i64, pitch: u8) -> bool {
        if at < self.watermark {
            return false;
        }
        self.resolve(at, false);
        self.watermark = at;
        if pitch > 127 || at < self.chart.targets[0].at - WINDOW_US || at > self.chart.end() {
            return true;
        }
        let indices = &self.chart.by_pitch[pitch as usize];
        let start =
            indices.partition_point(|&i| self.chart.targets[i].at < at.saturating_sub(WINDOW_US));
        let candidate = indices[start..]
            .iter()
            .copied()
            .take_while(|&i| self.chart.targets[i].at <= at.saturating_add(WINDOW_US))
            .filter(|&i| self.grades[i].is_none())
            .min_by_key(|&i| {
                (
                    (at - self.chart.targets[i].at).unsigned_abs(),
                    self.chart.targets[i].at,
                )
            });
        if let Some(i) = candidate {
            let error = at - self.chart.targets[i].at;
            let grade = match error.unsigned_abs() {
                0..=45_000 => Grade::Perfect,
                45_001..=90_000 => Grade::Great,
                _ => Grade::Good,
            };
            self.grades[i] = Some(grade);
            match grade {
                Grade::Perfect => self.stats.perfect += 1,
                Grade::Great => self.stats.great += 1,
                _ => self.stats.good += 1,
            }
            self.stats.early += u64::from(error < 0);
            self.stats.late += u64::from(error > 0);
            self.stats.timing_sum_us += error as i128;
            self.stats.histogram[((error + WINDOW_US) / 20_000).min(14) as usize] += 1;
            self.emit(Feedback {
                pitch,
                grade,
                error,
            });
        } else {
            self.stats.extras += 1;
            self.stats.streak = 0;
            self.raw -= 100;
            self.emit(Feedback {
                pitch,
                grade: Grade::Extra,
                error: 0,
            });
        }
        true
    }
    pub fn advance(&mut self, at: i64) {
        if at < self.watermark {
            return;
        }
        self.resolve(at, true);
        self.watermark = at;
    }
    fn resolve(&mut self, at: i64, inclusive: bool) {
        while let Some(group) = self.chart.groups.get(self.next_group) {
            if group.deadline > at || (!inclusive && group.deadline == at) {
                break;
            }
            let range = group.range.clone();
            let missed = range.clone().any(|i| self.grades[i].is_none());
            if missed {
                self.stats.streak = 0;
            }
            let m = multiplier(self.stats.streak) as i128;
            for i in range {
                let grade = if let Some(grade) = self.grades[i] {
                    grade
                } else {
                    self.grades[i] = Some(Grade::Miss);
                    self.stats.misses += 1;
                    self.emit(Feedback {
                        pitch: self.chart.targets[i].pitch,
                        grade: Grade::Miss,
                        error: 0,
                    });
                    Grade::Miss
                };
                self.raw += m * grade.points() as i128;
            }
            if !missed {
                self.stats.streak += 1;
                self.stats.longest_streak = self.stats.longest_streak.max(self.stats.streak);
            }
            self.next_group += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn judge(notes: &[(i64, u8)]) -> Judge {
        Judge::new(
            Chart::new(
                notes
                    .iter()
                    .map(|&(at, pitch)| Target { at, pitch })
                    .collect(),
            )
            .unwrap(),
        )
    }
    #[test]
    fn inclusive_timing_boundaries_and_symmetric_early_late() {
        for (error, expected) in [
            (0, 100),
            (45_000, 100),
            (45_001, 80),
            (90_000, 80),
            (90_001, 50),
            (150_000, 50),
            (150_001, 0),
        ] {
            for sign in [-1, 1] {
                let mut j = judge(&[(1_000_000, 60)]);
                j.attack(1_000_000 + sign * error, 60);
                j.advance(1_200_000);
                assert_eq!(j.stats.quality(), expected);
            }
        }
    }
    #[test]
    fn chord_order_unisons_partial_credit_and_extras() {
        for pitches in [[60, 64, 67], [67, 60, 64], [64, 67, 60]] {
            let mut j = judge(&[(0, 60), (0, 64), (0, 67), (0, 60)]);
            for p in pitches {
                assert!(j.attack(0, p));
            }
            j.advance(WINDOW_US);
            assert_eq!(j.score(), 1_000_000);
            assert_eq!(j.stats.streak, 1);
        }
        let mut j = judge(&[(0, 60), (0, 64), (0, 67)]);
        j.attack(0, 60);
        j.attack(100_000, 67);
        j.advance(WINDOW_US);
        assert_eq!(j.raw, 150);
        assert_eq!(j.stats.misses, 1);
        assert_eq!(j.stats.streak, 0);
        let mut j = judge(&[(0, 60)]);
        j.attack(-100_000, 61);
        j.attack(0, 60);
        j.advance(WINDOW_US);
        assert_eq!(j.score(), 0);
        assert_eq!(j.stats.accuracy(1), 50.0);
        assert!(!j.full_combo());
    }
    #[test]
    fn documented_multiplier_examples_and_no_score_farming() {
        for (miss, expected) in [(false, 1_000_000), (true, 785_714)] {
            let mut j = judge(&(0..12).map(|i| (i * 500_000, 60)).collect::<Vec<_>>());
            for i in 0..12 {
                if !(miss && i == 5) {
                    j.attack(i * 500_000, 60);
                }
            }
            j.advance(6_000_000);
            assert_eq!(j.score(), expected);
        }
        let mut j = judge(&[(0, 60), (500_000, 60)]);
        j.attack(0, 60);
        j.advance(700_000);
        assert_eq!(j.stats.misses, 1);
        let mut j = judge(&[(0, 60), (0, 64), (0, 67)]);
        for p in 0..128 {
            j.attack(0, p);
        }
        j.advance(WINDOW_US);
        assert_eq!(j.score(), 0);
        assert_eq!(j.stats.extras, 125);
    }
    #[test]
    fn repeated_pitch_nearest_ties_and_out_of_order() {
        let mut j = judge(&[(500_000, 60), (600_000, 60)]);
        j.attack(550_000, 60);
        assert!(j.grades[0].is_some());
        assert!(j.grades[1].is_none());
        j.attack(600_000, 60);
        j.advance(800_000);
        assert!(j.full_combo());
        assert!(!j.attack(400_000, 60));
    }
    #[test]
    fn replay_is_independent_of_frame_rate_and_stalls() {
        let events = [
            (980_000, 60),
            (1_035_000, 64),
            (1_070_000, 67),
            (1_470_000, 62),
            (1_750_000, 80),
        ];
        let mut results = Vec::new();
        for frame in [6_944, 16_667, 33_333, 600_000] {
            let mut j = judge(&[
                (1_000_000, 60),
                (1_000_000, 64),
                (1_000_000, 67),
                (1_500_000, 62),
                (2_000_000, 65),
            ]);
            let mut next = 0;
            for now in (0..=2_500_000).step_by(frame) {
                while next < events.len() && events[next].0 <= now {
                    j.attack(events[next].0, events[next].1);
                    next += 1;
                }
                j.advance(now);
            }
            j.advance(3_000_000);
            results.push((j.score(), j.stats.clone()));
        }
        assert!(results.windows(2).all(|w| w[0] == w[1]));
    }
    #[test]
    fn invalid_and_empty_charts_are_rejected_and_misses_finish() {
        assert!(Chart::new(vec![]).is_err());
        assert!(Chart::new(vec![Target { at: -1, pitch: 60 }]).is_err());
        assert!(Chart::new(vec![Target { at: 0, pitch: 128 }]).is_err());
        let mut j = judge(&[(0, 60)]);
        j.advance(WINDOW_US - 1);
        assert!(!j.finished());
        j.advance(WINDOW_US);
        assert!(j.finished());
        assert_eq!(j.stats.misses, 1);
        assert_eq!(j.score(), 0);
    }
}
