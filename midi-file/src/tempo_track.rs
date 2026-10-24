use midly::{MetaMessage, TrackEvent, TrackEventKind};
use std::{collections::HashMap, sync::Arc, time::Duration};

#[derive(Debug, Clone)]
pub struct TempoEvent {
    pub absolute_pulses: u64,
    pub timestamp: Duration,
    /// Tempo in microseconds per quarter note.
    pub tempo: u32,
}

#[derive(Debug, Clone)]
pub struct TempoTrack {
    pulses_per_quarter_note: u16,
    events: Arc<[TempoEvent]>,
}

impl TempoTrack {
    pub fn build(track_events: &[Vec<TrackEvent>], pulses_per_quarter_note: u16) -> TempoTrack {
        // This map will help us get rid of duplicate events if
        // the tempo is specified in every track (as is common).
        let mut tempo_events: HashMap<u64, TempoEvent> = HashMap::new();

        for events in track_events.iter() {
            let mut pulses: u64 = 0;
            for event in events.iter() {
                pulses += event.delta.as_int() as u64;

                if let TrackEventKind::Meta(MetaMessage::Tempo(t)) = &event.kind {
                    tempo_events.insert(
                        pulses,
                        TempoEvent {
                            absolute_pulses: pulses,
                            timestamp: Duration::ZERO,
                            tempo: t.as_int(),
                        },
                    );
                };
            }
        }

        let mut tempo_events: Vec<_> = tempo_events.into_values().collect();
        tempo_events.sort_by_key(|e| e.absolute_pulses);

        let mut previous_absolute_pulses = 0u64;
        let mut running_tempo = 500_000;
        let mut res = Duration::ZERO;

        for tempo_event in tempo_events.iter_mut() {
            let tempo_event_pulses = tempo_event.absolute_pulses;

            let relative_pulses = tempo_event_pulses - previous_absolute_pulses;

            res += pulse_to_duration(relative_pulses, running_tempo, pulses_per_quarter_note);

            tempo_event.timestamp = res;

            running_tempo = tempo_event.tempo;
            previous_absolute_pulses = tempo_event_pulses;
        }

        TempoTrack {
            pulses_per_quarter_note,
            events: tempo_events.into(),
        }
    }

    /// Invert the piecewise tempo map for a player timestamp. Quarter-note
    /// position remains continuous across tempo changes and independent of speed.
    pub fn seconds_to_quarters(&self, seconds: f64) -> f64 {
        if !seconds.is_finite() {
            return 0.0;
        }
        let index = self
            .events
            .partition_point(|e| e.timestamp.as_secs_f64() <= seconds);
        if let Some(event) = index.checked_sub(1).and_then(|i| self.events.get(i)) {
            event.absolute_pulses as f64 / f64::from(self.pulses_per_quarter_note)
                + (seconds - event.timestamp.as_secs_f64()) * 1_000_000.0
                    / f64::from(event.tempo.max(1))
        } else {
            let initial = self
                .events
                .first()
                .filter(|e| e.absolute_pulses == 0)
                .map(|e| e.tempo)
                .unwrap_or(500_000);
            seconds * 1_000_000.0 / f64::from(initial.max(1))
        }
    }

    pub fn tempo_event_for_pulses(&self, pulses: u64) -> Option<&TempoEvent> {
        let res = self
            .events
            .binary_search_by_key(&pulses, |e| e.absolute_pulses);

        let id = match res {
            Ok(id) => Some(id),
            Err(id) => id.checked_sub(1),
        };

        id.and_then(|id| self.events.get(id))
    }

    pub fn pulses_to_duration(&self, event_pulses: u64) -> Duration {
        let tempo_event = self.tempo_event_for_pulses(event_pulses);

        let (res, previous_absolute_pulses, tempo) = if let Some(event) = tempo_event {
            (event.timestamp, event.absolute_pulses, event.tempo)
        } else {
            // 120 BPM
            let default_tempo = 500_000;
            (Duration::ZERO, 0, default_tempo)
        };

        let delta_pulses = event_pulses - previous_absolute_pulses;
        res + pulse_to_duration(delta_pulses, tempo, self.pulses_per_quarter_note)
    }
}

fn pulse_to_duration(pulses: u64, tempo: u32, pulses_per_quarter_note: u16) -> Duration {
    let u_time = pulses as f64 / pulses_per_quarter_note as f64;
    // We floor only because Synthesia floors,
    // so if we want to test for timing regresions we have to do the same
    let time = (u_time * tempo as f64).floor() as u64;
    Duration::from_micros(time)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inverse_map_handles_default_initial_and_later_tempos() {
        let default = TempoTrack::build(&[], 480);
        assert_eq!(default.seconds_to_quarters(1.0), 2.0);
        let events = vec![vec![
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::Tempo(1_000_000.into())),
            },
            TrackEvent {
                delta: 960.into(),
                kind: TrackEventKind::Meta(MetaMessage::Tempo(500_000.into())),
            },
        ]];
        let map = TempoTrack::build(&events, 480);
        for (seconds, beats) in [
            (-1.0, -1.0),
            (0.0, 0.0),
            (1.0, 1.0),
            (2.0, 2.0),
            (2.5, 3.0),
            (3.0, 4.0),
        ] {
            assert_eq!(map.seconds_to_quarters(seconds), beats);
        }
        for pulses in [0, 240, 480, 959, 960, 1200, 1920] {
            assert!(
                (map.seconds_to_quarters(map.pulses_to_duration(pulses).as_secs_f64())
                    - pulses as f64 / 480.0)
                    .abs()
                    < 0.00001
            );
        }
    }
}
