use std::time::Duration;

use midi_file::midly::MidiMessage;
use neothesia_core::chords::{
    audition::{Audition, ReferenceEvent},
    training::Phase,
};

use super::{FreeplayScene, notes::control_pos};
use crate::{context::Context, output_manager::OutputConnection};

// Separate reference voice from the usual channel-1 live keyboard echo.
const REFERENCE_CHANNEL: u8 = 15;
#[derive(Clone, PartialEq, Eq)]
struct Prompt {
    kind: u8,
    index: usize,
    notes: Vec<u8>,
}
#[derive(Default)]
pub(super) struct PracticeAudio {
    enabled: bool,
    last_prompt: Option<Prompt>,
    audition: Audition,
    connection: Option<OutputConnection>,
}
impl PracticeAudio {
    fn send(&self, events: Vec<ReferenceEvent>) {
        if let Some(out) = &self.connection {
            for e in events {
                let message = match e {
                    ReferenceEvent::On(key) => MidiMessage::NoteOn {
                        key: key.into(),
                        vel: 80.into(),
                    },
                    ReferenceEvent::Off(key) => MidiMessage::NoteOff {
                        key: key.into(),
                        vel: 0.into(),
                    },
                };
                out.midi_event(REFERENCE_CHANNEL.into(), message);
            }
        }
    }
    fn stop(&mut self) {
        let events = self.audition.stop();
        self.send(events);
        self.connection = None;
    }
    pub(super) fn reset(&mut self) {
        self.stop();
        self.last_prompt = None;
    }
    fn play(&mut self, ctx: &Context, prompt: Prompt) {
        self.stop();
        self.connection = Some(ctx.output_manager.connection().clone());
        if let Some(out) = &self.connection {
            out.midi_event(
                REFERENCE_CHANNEL.into(),
                MidiMessage::Controller {
                    controller: 64.into(),
                    value: 0.into(),
                },
            );
            out.midi_event(
                REFERENCE_CHANNEL.into(),
                MidiMessage::ProgramChange { program: 0.into() },
            );
        }
        let events = self.audition.start(&prompt.notes);
        self.send(events);
        self.last_prompt = Some(prompt);
    }
}
impl Drop for PracticeAudio {
    fn drop(&mut self) {
        self.stop();
    }
}

impl FreeplayScene {
    fn practice_prompt(&self) -> Option<(Prompt, Phase)> {
        if let Some(s) = &self.note_practice.session {
            Some((
                Prompt {
                    kind: 0,
                    index: 0,
                    notes: vec![s.target().midi],
                },
                s.phase(),
            ))
        } else if let Some(s) = &self.training {
            s.exercise().map(|e| {
                (
                    Prompt {
                        kind: 1,
                        index: s.completed(),
                        notes: e.notes.to_vec(),
                    },
                    s.phase(),
                )
            })
        } else {
            None
        }
    }

    pub(super) fn update_practice_audio(&mut self, ctx: &Context, delta: Duration) {
        let events = self.practice_audio.audition.tick(delta);
        self.practice_audio.send(events);
        let Some((prompt, phase)) = self.practice_prompt() else {
            self.practice_audio.reset();
            return;
        };
        if !self.practice_audio.enabled {
            self.practice_audio.reset();
            return;
        }
        if phase == Phase::Playing && self.practice_audio.last_prompt.as_ref() != Some(&prompt) {
            self.practice_audio.play(ctx, prompt);
        }
    }

    pub(super) fn update_practice_audio_controls(&mut self, ctx: &Context) {
        if self.training.is_none() && self.note_practice.session.is_none() {
            return;
        }
        let width = ctx.window_state.logical_size.width;
        let base = if self.preview.is_some() { 125.0 } else { 80.0 };
        let index = self.note_control_count();
        let (x, y) = control_pos(width, index, base);
        let no_output = matches!(
            ctx.output_manager.connection(),
            OutputConnection::DummyOutput
        );
        let label = if !self.practice_audio.enabled {
            "Hear task: Off"
        } else if no_output {
            "Hear task: No out"
        } else {
            "Hear task: On"
        };
        if nuon::button()
            .id("hear_practice_task")
            .pos(x, y)
            .size(130.0, 30.0)
            .label(label)
            .color(if self.practice_audio.enabled {
                [45, 95, 145, 255]
            } else {
                [37, 35, 42, 255]
            })
            .border_radius([5.0; 4])
            .build(&mut self.nuon)
        {
            self.practice_audio.enabled = !self.practice_audio.enabled;
            self.practice_audio.reset();
        }
        let (x, y) = control_pos(width, index + 1, base);
        let can_replay = self.practice_audio.enabled
            && !no_output
            && self
                .practice_prompt()
                .is_some_and(|(_, phase)| phase != Phase::ReleaseBeforeStart);
        if nuon::button()
            .id("replay_practice_task")
            .pos(x, y)
            .size(130.0, 30.0)
            .label("Replay task")
            .font_color(if can_replay {
                [255, 255, 255, 255]
            } else {
                [255, 255, 255, 100]
            })
            .border_radius([5.0; 4])
            .build(&mut self.nuon)
            && can_replay
            && let Some((prompt, _)) = self.practice_prompt()
        {
            self.practice_audio.play(ctx, prompt);
        }
    }
}
