//! Short reference tones for practice. These events belong only on the output
//! path, never in live input state, recordings, or answer validation.
use std::time::Duration;

pub const REFERENCE_DURATION: Duration = Duration::from_millis(800);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceEvent {
    On(u8),
    Off(u8),
}
#[derive(Default)]
pub struct Audition {
    notes: Vec<u8>,
    remaining: Duration,
}
impl Audition {
    pub fn start(&mut self, notes: &[u8]) -> Vec<ReferenceEvent> {
        let mut events = self.stop();
        self.notes = notes.iter().copied().filter(|n| *n < 128).collect();
        self.notes.sort_unstable();
        self.notes.dedup();
        self.remaining = REFERENCE_DURATION;
        events.extend(self.notes.iter().map(|n| ReferenceEvent::On(*n)));
        events
    }
    pub fn tick(&mut self, delta: Duration) -> Vec<ReferenceEvent> {
        self.remaining = self.remaining.saturating_sub(delta);
        if self.remaining.is_zero() {
            self.stop()
        } else {
            Vec::new()
        }
    }
    pub fn stop(&mut self) -> Vec<ReferenceEvent> {
        self.remaining = Duration::ZERO;
        std::mem::take(&mut self.notes)
            .into_iter()
            .map(ReferenceEvent::Off)
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn note_and_chord_stop_at_deadline_once() {
        for notes in [vec![60], vec![60, 64, 67]] {
            let mut a = Audition::default();
            assert_eq!(
                a.start(&notes),
                notes
                    .iter()
                    .map(|n| ReferenceEvent::On(*n))
                    .collect::<Vec<_>>()
            );
            assert!(a.tick(Duration::from_millis(799)).is_empty());
            assert_eq!(
                a.tick(Duration::from_millis(1)),
                notes
                    .iter()
                    .map(|n| ReferenceEvent::Off(*n))
                    .collect::<Vec<_>>()
            );
            assert!(a.tick(REFERENCE_DURATION).is_empty());
        }
    }
    #[test]
    fn replay_cancellation_and_invalid_notes() {
        let mut a = Audition::default();
        a.start(&[60, 64, 67]);
        assert_eq!(
            a.start(&[62, 62, 255]),
            [
                ReferenceEvent::Off(60),
                ReferenceEvent::Off(64),
                ReferenceEvent::Off(67),
                ReferenceEvent::On(62)
            ]
        );
        assert_eq!(a.stop(), [ReferenceEvent::Off(62)]);
        assert!(a.stop().is_empty());
        assert!(a.tick(Duration::MAX).is_empty());
    }
}
