//! A time-indexed MIDI staff view. Positions use quarter-note beats, while the
//! player remains the only clock. This is a timed practice view, not engraving.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    Paged,
    #[default]
    Scrolling,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScoreWindow {
    pub start: f64,
    pub end: f64,
    pub cursor: f64,
}
impl ScoreWindow {
    pub fn new(beat: f64, span: f64, mode: ViewMode) -> Self {
        let beat = if beat.is_finite() { beat } else { 0.0 };
        let span = if span.is_finite() { span.max(1.0) } else { 8.0 };
        let start = match mode {
            ViewMode::Paged => (beat.max(0.0) / span).floor() * span,
            ViewMode::Scrolling => beat - span * 0.2,
        };
        Self {
            start,
            end: start + span,
            cursor: ((beat - start) / span).clamp(0.0, 1.0),
        }
    }
    pub fn fraction(&self, beat: f64) -> f64 {
        (beat - self.start) / (self.end - self.start)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ScoreNote {
    pub start: f64,
    pub end: f64,
    pub pitch: u8,
    pub track: usize,
}
impl ScoreNote {
    pub fn active(&self, beat: f64) -> bool {
        self.start <= beat && beat < self.end
    }
}
#[derive(Default)]
pub struct Score {
    notes: Vec<ScoreNote>,
    /// Monotone interval index keeps long held notes visible across page edges.
    prefix_end: Vec<f64>,
}
impl Score {
    pub fn new(mut notes: Vec<ScoreNote>) -> Self {
        notes.retain(|n| {
            n.pitch < 128
                && n.start.is_finite()
                && n.end.is_finite()
                && n.start >= 0.0
                && n.end > n.start
        });
        notes.sort_by(|a, b| {
            a.start
                .total_cmp(&b.start)
                .then(a.pitch.cmp(&b.pitch))
                .then(a.track.cmp(&b.track))
        });
        let mut end = 0.0_f64;
        let prefix_end = notes
            .iter()
            .map(|n| {
                end = end.max(n.end);
                end
            })
            .collect();
        Self { notes, prefix_end }
    }
    pub fn notes(&self) -> &[ScoreNote] {
        &self.notes
    }
    pub fn visible(&self, window: ScoreWindow) -> impl Iterator<Item = &ScoreNote> {
        let first = self.prefix_end.partition_point(|end| *end <= window.start);
        let last = self.notes.partition_point(|n| n.start < window.end);
        self.notes[first.min(last)..last]
            .iter()
            .filter(move |n| n.end > window.start)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteValue {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rhythm {
    pub value: NoteValue,
    pub dotted: bool,
}
impl Rhythm {
    /// Closest common written duration; exact performance duration stays in the
    /// time axis and duration rail. No invented ties, voices or quantized onset.
    pub fn nearest(beats: f64) -> Self {
        use NoteValue::*;
        let values = [
            (4.0, Whole, false),
            (3.0, Half, true),
            (2.0, Half, false),
            (1.5, Quarter, true),
            (1.0, Quarter, false),
            (0.75, Eighth, true),
            (0.5, Eighth, false),
            (0.375, Sixteenth, true),
            (0.25, Sixteenth, false),
            (0.1875, ThirtySecond, true),
            (0.125, ThirtySecond, false),
        ];
        let duration = if beats.is_finite() {
            beats.max(0.001)
        } else {
            1.0
        };
        let (_, value, dotted) = values
            .into_iter()
            .min_by(|a, b| {
                (duration / a.0)
                    .ln()
                    .abs()
                    .total_cmp(&(duration / b.0).ln().abs())
            })
            .unwrap();
        Self { value, dotted }
    }
    pub fn flags(self) -> u8 {
        match self.value {
            NoteValue::Eighth => 1,
            NoteValue::Sixteenth => 2,
            NoteValue::ThirtySecond => 3,
            _ => 0,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paged_notes_stay_put_and_cursor_moves_then_page_turns() {
        let a = ScoreWindow::new(2.0, 8.0, ViewMode::Paged);
        let b = ScoreWindow::new(6.0, 8.0, ViewMode::Paged);
        assert_eq!(a.start, b.start);
        assert_eq!(a.fraction(4.0), b.fraction(4.0));
        assert!(a.cursor < b.cursor);
        let c = ScoreWindow::new(8.0, 8.0, ViewMode::Paged);
        assert_eq!(c.start, 8.0);
        assert_eq!(c.cursor, 0.0);
        assert_eq!(ScoreWindow::new(-2.0, 8.0, ViewMode::Paged).cursor, 0.0);
    }
    #[test]
    fn scrolling_cursor_is_fixed_and_seeking_has_no_cached_clock() {
        let a = ScoreWindow::new(2.0, 8.0, ViewMode::Scrolling);
        let b = ScoreWindow::new(6.0, 8.0, ViewMode::Scrolling);
        assert!((a.cursor - 0.2).abs() < 1e-10);
        assert!((b.cursor - 0.2).abs() < 1e-10);
        assert!(b.fraction(4.0) < a.fraction(4.0));
        assert_eq!(a, ScoreWindow::new(2.0, 8.0, ViewMode::Scrolling));
        assert!(
            ScoreWindow::new(f64::NAN, f64::NAN, ViewMode::Paged)
                .start
                .is_finite()
        );
    }
    #[test]
    fn visible_intervals_include_held_notes_and_exclude_edges() {
        let s = Score::new(vec![
            ScoreNote {
                start: 0.0,
                end: 20.0,
                pitch: 48,
                track: 0,
            },
            ScoreNote {
                start: 1.0,
                end: 2.0,
                pitch: 60,
                track: 1,
            },
            ScoreNote {
                start: 8.0,
                end: 9.0,
                pitch: 64,
                track: 0,
            },
            ScoreNote {
                start: 16.0,
                end: 17.0,
                pitch: 67,
                track: 0,
            },
            ScoreNote {
                start: 3.0,
                end: 3.0,
                pitch: 70,
                track: 0,
            },
        ]);
        let v: Vec<_> = s
            .visible(ScoreWindow::new(8.0, 8.0, ViewMode::Paged))
            .collect();
        assert_eq!(v.iter().map(|n| n.pitch).collect::<Vec<_>>(), [48, 64]);
        assert!(v[0].active(8.0));
        assert!(!v[1].active(9.0));
        assert_eq!(
            s.visible(ScoreWindow::new(0.0, 8.0, ViewMode::Paged))
                .count(),
            2
        );
    }
    #[test]
    fn rhythm_symbols_preserve_common_values_and_dots() {
        assert_eq!(
            Rhythm::nearest(3.0),
            Rhythm {
                value: NoteValue::Half,
                dotted: true
            }
        );
        assert_eq!(Rhythm::nearest(0.49).flags(), 1);
        assert_eq!(Rhythm::nearest(0.25).flags(), 2);
        assert_eq!(Rhythm::nearest(0.125).flags(), 3);
        assert_eq!(Rhythm::nearest(8.0).value, NoteValue::Whole);
        assert_eq!(Rhythm::nearest(f64::NAN).value, NoteValue::Quarter);
    }
}
