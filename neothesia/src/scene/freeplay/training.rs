use neothesia_core::chords::training::{EXERCISES, Phase, TrainingSession};

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
                [45, 95, 145, 255]
            } else {
                [37, 35, 42, 255]
            })
            .border_radius([5.0; 4])
            .build(&mut self.nuon)
        {
            self.training = if self.training.is_some() {
                None
            } else {
                Some(TrainingSession::new(&self.live_notes.notes()))
            };
        }
        if self.training.is_none() {
            return;
        }
        if nuon::button()
            .id("restart_chord_training")
            .pos(300.0, controls_y)
            .size(100.0, 30.0)
            .label("Restart")
            .border_radius([5.0; 4])
            .build(&mut self.nuon)
        {
            self.training = Some(TrainingSession::new(&self.live_notes.notes()));
        }

        let session = self.training.as_ref().unwrap();
        let width = ctx.window_state.logical_size.width;
        let top = controls_y + 45.0;
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
            .color([37, 35, 42, 240])
            .border_radius([5.0; 4])
            .build(&mut self.nuon);
        nuon::label()
            .pos(15.0, top + 5.0)
            .size((width - 30.0).max(0.0), 90.0)
            .text(text)
            .font_size(20.0)
            .text_justify(nuon::TextAlign::Center)
            .build(&mut self.nuon);

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
