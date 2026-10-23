use tangent_core::chords::training::{EXERCISES, Phase, TrainingSession};

use super::FreeplayScene;
use crate::context::Context;

impl FreeplayScene {
    pub(super) fn update_training_ui(&mut self, ctx: &Context) {
        let controls_y = if self.preview.is_some() { 85.0 } else { 40.0 };
        if nuon::button()
            .id("freeplay_training")
            .pos(150.0, controls_y)
            .size(140.0, 30.0)
            .label(if self.training.is_some() {
                "Training: On"
            } else {
                "Training: Off"
            })
            .color(if self.training.is_some() {
                self.nuon.theme.selected
            } else {
                self.nuon.theme.button
            })
            .border_radius([8.0; 4])
            .build(&mut self.nuon)
        {
            self.practice_audio.reset();
            self.training = if self.training.is_some() {
                None
            } else {
                Some(TrainingSession::new(&self.live_notes.notes()))
            };
            if self.training.is_some() {
                self.note_practice.session = None;
            }
        }
        if self.training.is_none() {
            return;
        }
        if nuon::button()
            .id("restart_chord_training")
            .pos(300.0, controls_y)
            .size(100.0, 30.0)
            .label("Restart")
            .border_radius([8.0; 4])
            .build(&mut self.nuon)
        {
            self.practice_audio.reset();
            self.training = Some(TrainingSession::new(&self.live_notes.notes()));
        }

        let session = self.training.as_ref().unwrap();
        let width = ctx.window_state.logical_size.width;
        let top = self.note_controls_bottom(ctx);
        let text = if let Some(exercise) = session.exercise() {
            let feedback = match session.phase() {
                Phase::ReleaseBeforeStart => "Release keys and sustain pedal to begin.",
                Phase::Correct => "Correct! Release keys and pedal to continue.",
                _ => "Play the chord and hold for 0.3s. Any octave/inversion.",
            };
            format!(
                "Chord {} / {}: {}\n{}\n{}",
                session.completed() + 1,
                EXERCISES.len(),
                exercise.symbol,
                exercise.note_labels(),
                feedback
            )
        } else {
            format!(
                "Training complete! {} / {} chords\nPress Restart to practice again.",
                EXERCISES.len(),
                EXERCISES.len()
            )
        };
        nuon::quad()
            .pos(10.0, top)
            .size((width - 20.0).max(0.0), 100.0)
            .color(self.nuon.theme.surface)
            .border_radius([8.0; 4])
            .build(&mut self.nuon);
        let lines: Vec<_> = text.lines().collect();
        let title = if let Some(exercise) = session.exercise() {
            exercise.symbol.to_owned()
        } else {
            "Well played".to_owned()
        };
        for (y, height, size, value, muted) in [
            (
                7.0,
                16.0,
                12.0,
                format!(
                    "CHORD PRACTICE  ·  {} / {}",
                    session.completed().saturating_add(1).min(EXERCISES.len()),
                    EXERCISES.len()
                ),
                true,
            ),
            (25.0, 32.0, 30.0, title, false),
            (
                59.0,
                16.0,
                14.0,
                lines.get(1).unwrap_or(&"").to_string(),
                true,
            ),
            (
                80.0,
                15.0,
                12.0,
                lines.get(2).unwrap_or(&"").to_string(),
                true,
            ),
        ] {
            nuon::label()
                .pos(15.0, top + y)
                .size((width - 30.0).max(0.0), height)
                .text(value)
                .font_size(size)
                .text_justify(nuon::TextAlign::Center)
                .color(if muted {
                    self.nuon.theme.muted
                } else {
                    nuon::Color::WHITE
                })
                .build(&mut self.nuon);
        }

        // Suggested voicing markers; never modify played/preview key state.
        if let Some(exercise) = session.exercise() {
            let layout = self.keyboard.layout();
            for note in exercise.notes {
                if !layout.range.contains(*note) {
                    continue;
                }
                let key = &layout.keys[(*note - layout.range.start()) as usize];
                nuon::quad()
                    .pos(key.x() + 2.0, self.keyboard.pos().y - 7.0)
                    .size((key.width() - 4.0).max(1.0), 5.0)
                    .color(if session.phase() == Phase::Correct {
                        [80, 210, 120, 255]
                    } else {
                        [255, 195, 65, 255]
                    })
                    .build(&mut self.nuon);
            }
        }
    }
}
