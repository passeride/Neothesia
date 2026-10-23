use midi_file::midly::{self, MidiMessage, live::LiveEvent};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use winit::event_loop::EventLoopProxy;

use crate::TangentEvent;

#[derive(Clone, Copy)]
pub struct TimedInput {
    pub at: Instant,
    pub channel: u8,
    pub message: MidiMessage,
}
#[derive(Default)]
struct ScoringInput {
    enabled: bool,
    overflow: bool,
    pending: Vec<TimedInput>,
}
impl ScoringInput {
    fn push(&mut self, channel: u8, message: MidiMessage) {
        if !self.enabled {
            return;
        }
        if self.pending.len() >= 8192 {
            self.overflow = true;
            return;
        }
        self.pending.push(TimedInput {
            at: Instant::now(),
            channel,
            message,
        });
    }
}

pub struct InputManager {
    scoring: Arc<Mutex<ScoringInput>>,
    input: Option<midi_io::MidiInputManager>,
    tx: EventLoopProxy<TangentEvent>,
    current_connection: Option<(midi_io::MidiInputPort, midi_io::MidiInputConnection)>,
}

impl InputManager {
    pub fn new(tx: EventLoopProxy<TangentEvent>) -> Self {
        let input = midi_io::MidiInputManager::new()
            .map_err(|error| log::warn!("MIDI input unavailable: {error}"))
            .ok();
        Self {
            scoring: Arc::default(),
            input,
            tx,
            current_connection: None,
        }
    }

    pub fn enable_scoring(&self, enabled: bool) {
        let mut scoring = self.scoring.lock().unwrap();
        scoring.enabled = enabled;
        scoring.pending.clear();
        scoring.overflow = false;
    }

    pub fn queue_scoring_input(&self, channel: u8, message: MidiMessage) {
        self.scoring.lock().unwrap().push(channel, message);
    }

    /// Timestamp inside the same short lock as callback insertion. No callback
    /// can later enqueue an attack older than this watermark.
    pub fn take_scoring_input(&self) -> (Vec<TimedInput>, Instant, bool) {
        let mut scoring = self.scoring.lock().unwrap();
        let now = Instant::now();
        let events = std::mem::take(&mut scoring.pending);
        let overflow = std::mem::take(&mut scoring.overflow);
        (events, now, overflow)
    }

    pub fn connected_name(&self) -> Option<String> {
        self.current_connection
            .as_ref()
            .map(|(port, _)| port.to_string())
    }

    pub fn inputs(&self) -> Vec<midi_io::MidiInputPort> {
        self.input
            .as_ref()
            .map(|input| input.inputs())
            .unwrap_or_default()
    }

    pub fn connect_input(&mut self, port: midi_io::MidiInputPort) {
        let tx = self.tx.clone();
        let scoring = self.scoring.clone();

        if let Some((current, _)) = self.current_connection.as_ref()
            && current == &port
        {
            return;
        }

        // Close the connection first, as Windows does not like it when we hold 2 connections
        self.current_connection = None;

        self.current_connection = midi_io::MidiInputManager::connect_input(port, move |message| {
            let Ok(event) = LiveEvent::parse(message) else {
                return;
            };

            if let LiveEvent::Midi { channel, message } = event {
                scoring.lock().unwrap().push(channel.as_int(), message);
                match message {
                    // Some keyboards send NoteOn event with vel 0 instead of NoteOff
                    midly::MidiMessage::NoteOn { key, vel } if vel == 0 => {
                        tx.send_event(TangentEvent::MidiInput {
                            channel: channel.as_int(),
                            message: MidiMessage::NoteOff { key, vel },
                        })
                        .ok();
                    }
                    message => {
                        tx.send_event(TangentEvent::MidiInput {
                            channel: channel.as_int(),
                            message,
                        })
                        .ok();
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod scoring_queue_tests {
    use super::*;
    #[test]
    fn inactive_queue_is_empty_and_overflow_is_explicit_without_unbounded_growth() {
        let mut q = ScoringInput::default();
        let message = MidiMessage::NoteOn {
            key: 60.into(),
            vel: 100.into(),
        };
        q.push(0, message);
        assert!(q.pending.is_empty());
        q.enabled = true;
        for _ in 0..8193 {
            q.push(0, message);
        }
        assert_eq!(q.pending.len(), 8192);
        assert!(q.overflow);
        assert!(q.pending.windows(2).all(|w| w[0].at <= w[1].at));
    }
}
