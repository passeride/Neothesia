//! Staff positions and single-note reading practice, using scientific pitch (C4=60).
use std::time::Duration;

use super::{
    Spelling, note_name,
    training::{HOLD_TIME, Phase},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clef {
    Treble,
    Bass,
}
impl Clef {
    pub fn label(self) -> &'static str {
        match self {
            Self::Treble => "Treble (G clef)",
            Self::Bass => "Bass (F clef)",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PracticeRange {
    Treble,
    Bass,
    Both,
}
impl PracticeRange {
    pub fn next(self) -> Self {
        match self {
            Self::Treble => Self::Bass,
            Self::Bass => Self::Both,
            Self::Both => Self::Treble,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Treble => "Treble",
            Self::Bass => "Bass",
            Self::Both => "Both clefs",
        }
    }
}

pub fn pitch_label(midi: u8) -> String {
    format!(
        "{}{}",
        note_name(midi, Spelling::Sharps),
        i16::from(midi) / 12 - 1
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StaffPitch {
    /// Half staff spaces above the bottom line (E4 for treble, G2 for bass).
    pub step: i16,
    pub sharp: bool,
}

pub fn staff_pitch(midi: u8, clef: Clef) -> StaffPitch {
    // Black keys are spelled as sharps; each maps to the preceding white key.
    const LETTER: [i16; 12] = [0, 0, 1, 1, 2, 3, 3, 4, 4, 5, 5, 6];
    let pc = (midi % 12) as usize;
    let diatonic = (i16::from(midi) / 12 - 1) * 7 + LETTER[pc];
    let bottom = match clef {
        Clef::Treble => 4 * 7 + 2,
        Clef::Bass => 2 * 7 + 4,
    };
    StaffPitch {
        step: diatonic - bottom,
        sharp: matches!(pc, 1 | 3 | 6 | 8 | 10),
    }
}

/// Ledger lines are spaced on even steps, beyond the five staff lines (0..8).
pub fn ledger_steps(step: i16) -> Vec<i16> {
    if step < -1 {
        (step..=-2).filter(|s| s % 2 == 0).collect()
    } else if step > 9 {
        (10..=step).filter(|s| s % 2 == 0).collect()
    } else {
        Vec::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoteTarget {
    pub midi: u8,
    pub clef: Clef,
}

pub const ROUND_LENGTH: usize = 20;
pub struct NoteTraining {
    targets: Vec<NoteTarget>,
    index: usize,
    phase: Phase,
    matching_for: Duration,
}

fn shuffled_pool(clef: Clef, chromatic: bool, rng: &mut u64) -> Vec<NoteTarget> {
    let (lo, hi) = match clef {
        Clef::Treble => (60, 84),
        Clef::Bass => (36, 60),
    };
    let mut pool: Vec<_> = (lo..=hi)
        .filter(|n| chromatic || !staff_pitch(*n, clef).sharp)
        .map(|midi| NoteTarget { midi, clef })
        .collect();
    for i in (1..pool.len()).rev() {
        *rng ^= *rng << 13;
        *rng ^= *rng >> 7;
        *rng ^= *rng << 17;
        pool.swap(i, (*rng as usize) % (i + 1));
    }
    pool
}

impl NoteTraining {
    /// Seed is supplied by the UI; reproducible seeds make sequence tests stable.
    pub fn new(range: PracticeRange, chromatic: bool, seed: u64, current_notes: &[u8]) -> Self {
        let mut rng = if seed == 0 { 1 } else { seed };
        let treble = shuffled_pool(Clef::Treble, chromatic, &mut rng);
        let bass = shuffled_pool(Clef::Bass, chromatic, &mut rng);
        let mut targets: Vec<NoteTarget> = Vec::new();
        let mut treble_i = 0;
        let mut bass_i = 0;
        for i in 0..ROUND_LENGTH {
            let clef = match range {
                PracticeRange::Treble => Clef::Treble,
                PracticeRange::Bass => Clef::Bass,
                PracticeRange::Both => {
                    if i % 2 == 0 {
                        Clef::Treble
                    } else {
                        Clef::Bass
                    }
                }
            };
            let (pool, index) = match clef {
                Clef::Treble => (&treble, &mut treble_i),
                Clef::Bass => (&bass, &mut bass_i),
            };
            // Middle C belongs to both pools. Avoid the same key twice in a row.
            if targets
                .last()
                .is_some_and(|p| p.midi == pool[*index % pool.len()].midi)
            {
                *index += 1;
            }
            targets.push(pool[*index % pool.len()]);
            *index += 1;
        }
        Self {
            targets,
            index: 0,
            phase: if current_notes.is_empty() {
                Phase::Playing
            } else {
                Phase::ReleaseBeforeStart
            },
            matching_for: Duration::ZERO,
        }
    }
    pub fn target(&self) -> Option<NoteTarget> {
        self.targets.get(self.index).copied()
    }
    pub fn completed(&self) -> usize {
        self.index
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn update(&mut self, notes: &[u8], delta: Duration) {
        match self.phase {
            Phase::Finished => {}
            Phase::ReleaseBeforeStart => {
                if notes.is_empty() {
                    self.phase = Phase::Playing;
                }
            }
            Phase::Correct => {
                if notes.is_empty() {
                    self.index += 1;
                    self.matching_for = Duration::ZERO;
                    self.phase = if self.index == self.targets.len() {
                        Phase::Finished
                    } else {
                        Phase::Playing
                    };
                }
            }
            Phase::Playing => {
                // Exact MIDI pitch and exactly one sounding key: octave and extras matter.
                if notes == [self.target().unwrap().midi] {
                    self.matching_for = self.matching_for.saturating_add(delta);
                    if self.matching_for >= HOLD_TIME {
                        self.phase = Phase::Correct;
                    }
                } else {
                    self.matching_for = Duration::ZERO;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn staff_reference_pitches_and_middle_c() {
        for (midi, clef, step) in [
            (64, Clef::Treble, 0),
            (67, Clef::Treble, 2),
            (77, Clef::Treble, 8),
            (60, Clef::Treble, -2),
            (43, Clef::Bass, 0),
            (57, Clef::Bass, 8),
            (60, Clef::Bass, 10),
            (36, Clef::Bass, -4),
        ] {
            assert_eq!(staff_pitch(midi, clef), StaffPitch { step, sharp: false });
        }
        assert_eq!(
            staff_pitch(61, Clef::Treble),
            StaffPitch {
                step: -2,
                sharp: true
            }
        );
        assert_eq!(ledger_steps(-5), [-4, -2]);
        assert_eq!(ledger_steps(13), [10, 12]);
        assert!(ledger_steps(9).is_empty());
        assert!(ledger_steps(-1).is_empty());
        assert_eq!(pitch_label(60), "C4");
        assert_eq!(pitch_label(0), "C-1");
    }
    #[test]
    fn decks_cover_selected_registers_and_balanced_clefs() {
        for range in [
            PracticeRange::Treble,
            PracticeRange::Bass,
            PracticeRange::Both,
        ] {
            for seed in 0..100 {
                let s = NoteTraining::new(range, false, seed, &[]);
                assert_eq!(s.targets.len(), ROUND_LENGTH);
                assert!(s.targets.windows(2).all(|p| p[0].midi != p[1].midi));
                for t in &s.targets {
                    assert!(!staff_pitch(t.midi, t.clef).sharp);
                    assert!(match t.clef {
                        Clef::Treble => (60..=84).contains(&t.midi),
                        Clef::Bass => (36..=60).contains(&t.midi),
                    });
                    if range != PracticeRange::Both {
                        assert_eq!(
                            t.clef,
                            if range == PracticeRange::Treble {
                                Clef::Treble
                            } else {
                                Clef::Bass
                            }
                        );
                    }
                }
                if range == PracticeRange::Both {
                    assert_eq!(
                        s.targets.iter().filter(|t| t.clef == Clef::Bass).count(),
                        10
                    );
                }
            }
        }
        let a = NoteTraining::new(PracticeRange::Both, true, 42, &[]);
        assert!(a.targets.iter().any(|t| staff_pitch(t.midi, t.clef).sharp));
        assert_eq!(
            a.targets,
            NoteTraining::new(PracticeRange::Both, true, 42, &[]).targets
        );
        assert_ne!(
            a.targets,
            NoteTraining::new(PracticeRange::Both, true, 43, &[]).targets
        );
    }
    #[test]
    fn exact_octave_extras_and_hold_reset() {
        let mut s = NoteTraining::new(PracticeRange::Treble, false, 1, &[]);
        let n = s.target().unwrap().midi;
        for wrong in [vec![n - 12], vec![n, n + 1], vec![]] {
            s.update(&[n], Duration::from_millis(200));
            s.update(&wrong, HOLD_TIME);
            s.update(&[n], Duration::from_millis(100));
            assert_eq!(s.phase(), Phase::Playing);
            s.update(&[], Duration::ZERO);
        }
        s.update(&[n], HOLD_TIME);
        assert_eq!(s.phase(), Phase::Correct);
        s.update(&[n], HOLD_TIME);
        assert_eq!(s.completed(), 0);
        s.update(&[], Duration::ZERO);
        assert_eq!(s.completed(), 1);
    }
    #[test]
    fn release_gating_completion_and_restart() {
        let mut s = NoteTraining::new(PracticeRange::Bass, true, 3, &[60]);
        let n = s.target().unwrap().midi;
        s.update(&[n], HOLD_TIME);
        assert_eq!(s.phase(), Phase::ReleaseBeforeStart);
        s.update(&[], Duration::ZERO);
        for _ in 0..ROUND_LENGTH {
            let n = s.target().unwrap().midi;
            s.update(&[n], HOLD_TIME);
            assert_eq!(s.phase(), Phase::Correct);
            s.update(&[], Duration::ZERO);
        }
        assert_eq!(s.phase(), Phase::Finished);
        assert!(s.target().is_none());
        s.update(&[60], HOLD_TIME);
        assert_eq!(s.completed(), ROUND_LENGTH);
        assert_eq!(
            NoteTraining::new(PracticeRange::Bass, false, 4, &[]).completed(),
            0
        );
    }
}
