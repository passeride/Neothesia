//! A short beginner chord sequence. Answers compare pitch classes, not the
//! recognizer's first symbol: harmonically equivalent names must not fail a task.
use std::time::Duration;

use super::{Spelling, note_name};

pub const HOLD_TIME: Duration = Duration::from_millis(300);

pub struct Exercise {
    pub symbol: &'static str,
    /// Suggested middle-register voicing; other octaves/inversions are accepted.
    pub notes: &'static [u8],
    pub spelling: Spelling,
}

macro_rules! exercise {
    ($name:literal, [$($note:literal),*]) => {
        Exercise { symbol: $name, notes: &[$($note),*], spelling: Spelling::Sharps }
    };
}

/// A curated practice order, not a statistical ranking of chord frequency.
pub const EXERCISES: &[Exercise] = &[
    exercise!("C", [60, 64, 67]),
    exercise!("F", [65, 69, 72]),
    exercise!("G", [67, 71, 74]),
    exercise!("Am", [57, 60, 64]),
    exercise!("Dm", [62, 65, 69]),
    exercise!("Em", [64, 67, 71]),
    exercise!("D", [62, 66, 69]),
    exercise!("A", [57, 61, 64]),
    exercise!("E", [64, 68, 71]),
    exercise!("Bm", [59, 62, 66]),
    Exercise {
        symbol: "Bb",
        notes: &[58, 62, 65],
        spelling: Spelling::Flats,
    },
    Exercise {
        symbol: "Eb",
        notes: &[63, 67, 70],
        spelling: Spelling::Flats,
    },
    exercise!("G7", [55, 59, 62, 65]),
    exercise!("D7", [62, 66, 69, 72]),
    exercise!("A7", [57, 61, 64, 67]),
    exercise!("E7", [64, 68, 71, 74]),
    exercise!("Cmaj7", [60, 64, 67, 71]),
    exercise!("Am7", [57, 60, 64, 67]),
    exercise!("Dm7", [62, 65, 69, 72]),
    exercise!("Fmaj7", [65, 69, 72, 76]),
];

impl Exercise {
    pub fn note_labels(&self) -> String {
        self.notes
            .iter()
            .map(|n| format!("{}{}", note_name(*n, self.spelling), i16::from(*n) / 12 - 1))
            .collect::<Vec<_>>()
            .join(" - ")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    ReleaseBeforeStart,
    Playing,
    Correct,
    Finished,
}

pub struct TrainingSession {
    index: usize,
    phase: Phase,
    matching_for: Duration,
}

fn pitch_classes(notes: &[u8]) -> Vec<u8> {
    let mut classes: Vec<_> = notes
        .iter()
        .filter(|n| **n <= 127)
        .map(|n| n % 12)
        .collect();
    classes.sort_unstable();
    classes.dedup();
    classes
}

impl TrainingSession {
    pub fn new(current_notes: &[u8]) -> Self {
        Self {
            index: 0,
            phase: if current_notes.is_empty() {
                Phase::Playing
            } else {
                Phase::ReleaseBeforeStart
            },
            matching_for: Duration::ZERO,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn completed(&self) -> usize {
        self.index
    }
    pub fn exercise(&self) -> Option<&'static Exercise> {
        EXERCISES.get(self.index)
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
                    self.phase = if self.index == EXERCISES.len() {
                        Phase::Finished
                    } else {
                        Phase::Playing
                    };
                }
            }
            Phase::Playing => {
                if pitch_classes(notes) == pitch_classes(self.exercise().unwrap().notes) {
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
    fn inversions_and_octave_doubles_are_accepted_after_hold() {
        let mut s = TrainingSession::new(&[]);
        s.update(&[64, 67, 72, 79], Duration::from_millis(150));
        assert_eq!(s.phase(), Phase::Playing);
        s.update(&[64, 67, 72, 79], Duration::from_millis(150));
        assert_eq!(s.phase(), Phase::Correct);
        assert_eq!(s.completed(), 0);
        s.update(&[64], HOLD_TIME);
        assert_eq!(s.phase(), Phase::Correct);
        s.update(&[], Duration::ZERO);
        assert_eq!(s.completed(), 1);
        assert_eq!(s.exercise().unwrap().symbol, "F");
    }
    #[test]
    fn incomplete_wrong_and_extra_notes_reset_hold() {
        let mut s = TrainingSession::new(&[]);
        for notes in [&[60, 64][..], &[60, 63, 67], &[60, 64, 67, 69]] {
            s.update(EXERCISES[0].notes, Duration::from_millis(200));
            s.update(notes, HOLD_TIME);
            assert_eq!(s.phase(), Phase::Playing);
            s.update(EXERCISES[0].notes, Duration::from_millis(100));
            assert_eq!(s.phase(), Phase::Playing);
            s.update(&[], Duration::ZERO);
        }
    }
    #[test]
    fn starting_with_sounding_notes_requires_release() {
        let mut s = TrainingSession::new(EXERCISES[0].notes);
        s.update(EXERCISES[0].notes, HOLD_TIME);
        assert_eq!(s.phase(), Phase::ReleaseBeforeStart);
        s.update(&[], Duration::ZERO);
        assert_eq!(s.phase(), Phase::Playing);
        s.update(EXERCISES[0].notes, HOLD_TIME);
        assert_eq!(s.phase(), Phase::Correct);
    }
    #[test]
    fn whole_series_finishes_and_restart_resets_progress() {
        let mut s = TrainingSession::new(&[]);
        for (i, e) in EXERCISES.iter().enumerate() {
            assert_eq!(s.completed(), i);
            s.update(e.notes, HOLD_TIME);
            assert_eq!(s.phase(), Phase::Correct, "{}", e.symbol);
            s.update(&[], Duration::ZERO);
        }
        assert_eq!(s.phase(), Phase::Finished);
        assert!(s.exercise().is_none());
        s.update(EXERCISES[0].notes, HOLD_TIME);
        assert_eq!(s.completed(), EXERCISES.len());
        s = TrainingSession::new(&[]);
        assert_eq!(s.completed(), 0);
    }
    #[test]
    fn displayed_notes_use_scientific_octaves_and_flats() {
        assert_eq!(EXERCISES[0].note_labels(), "C4 - E4 - G4");
        assert_eq!(EXERCISES[10].note_labels(), "Bb3 - D4 - F4");
    }
    #[test]
    fn sustained_notes_block_advance_until_pedal_release() {
        let mut live = super::super::live::LiveNotes::default();
        let mut s = TrainingSession::new(&[]);
        live.controller(0, 64, 127);
        for n in EXERCISES[0].notes {
            live.note_on(0, *n, 100);
        }
        s.update(&live.notes(), HOLD_TIME);
        for n in EXERCISES[0].notes {
            live.note_off(0, *n);
        }
        s.update(&live.notes(), HOLD_TIME);
        assert_eq!(s.phase(), Phase::Correct);
        live.controller(0, 64, 0);
        s.update(&live.notes(), Duration::ZERO);
        assert_eq!(s.exercise().unwrap().symbol, "F");
    }
}
