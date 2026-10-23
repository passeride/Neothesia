use std::time::{SystemTime, UNIX_EPOCH};

use neothesia_core::{
    chords::{
        Spelling,
        notes::{
            Clef, NoteTraining, PracticeRange, chord_staff_notes, ledger_steps, pitch_label,
            staff_pitch,
        },
        training::Phase,
    },
    render::{QuadInstance, QuadRenderer, TextRenderer},
};

use super::FreeplayScene;
use crate::{context::Context, icons};

pub(super) struct NotePractice {
    pub show_notes: bool,
    pub session: Option<NoteTraining>,
    range: PracticeRange,
    chromatic: bool,
    hint: bool,
    zoom_percent: u16,
}
impl Default for NotePractice {
    fn default() -> Self {
        Self {
            show_notes: false,
            session: None,
            range: PracticeRange::Both,
            chromatic: false,
            hint: false,
            zoom_percent: 100,
        }
    }
}

pub(super) fn control_pos(width: f32, index: usize, y: f32) -> (f32, f32) {
    let columns = ((width - 20.0) / 140.0).floor().max(1.0) as usize;
    (
        10.0 + (index % columns) as f32 * 140.0,
        y + (index / columns) as f32 * 35.0,
    )
}

fn toggle(
    ui: &mut nuon::Ui,
    id: &'static str,
    pos: (f32, f32),
    label: String,
    active: bool,
) -> bool {
    nuon::button()
        .id(id)
        .pos(pos.0, pos.1)
        .size(130.0, 30.0)
        .label(label)
        .color(if active {
            [45, 95, 145, 255]
        } else {
            [37, 35, 42, 255]
        })
        .border_radius([5.0; 4])
        .build(ui)
}

impl FreeplayScene {
    fn restart_note_training(&mut self) {
        self.practice_audio.reset();
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1);
        self.note_practice.session = Some(NoteTraining::new(
            self.note_practice.range,
            self.note_practice.chromatic,
            seed,
            &self.live_notes.notes(),
        ));
        self.training = None;
    }

    pub(super) fn note_controls_bottom(&self, ctx: &Context) -> f32 {
        let base = if self.preview.is_some() { 125.0 } else { 80.0 };
        let count = self.note_control_count()
            + if self.training.is_some() || self.note_practice.session.is_some() {
                2
            } else {
                0
            };
        let last = count - 1;
        control_pos(ctx.window_state.logical_size.width, last, base).1 + 40.0
    }

    pub(super) fn note_control_count(&self) -> usize {
        if self.note_practice.session.is_some() {
            7
        } else if self.note_practice.show_notes || self.training.is_some() {
            3
        } else {
            2
        }
    }

    pub(super) fn update_note_controls(&mut self, ctx: &Context) {
        let base = if self.preview.is_some() { 125.0 } else { 80.0 };
        let width = ctx.window_state.logical_size.width;
        if toggle(
            &mut self.nuon,
            "live_note_display",
            control_pos(width, 0, base),
            format!(
                "Notes: {}",
                if self.note_practice.show_notes {
                    "On"
                } else {
                    "Off"
                }
            ),
            self.note_practice.show_notes,
        ) {
            self.note_practice.show_notes = !self.note_practice.show_notes;
        }
        if toggle(
            &mut self.nuon,
            "note_training",
            control_pos(width, 1, base),
            format!(
                "Note practice: {}",
                if self.note_practice.session.is_some() {
                    "On"
                } else {
                    "Off"
                }
            ),
            self.note_practice.session.is_some(),
        ) {
            if self.note_practice.session.is_some() {
                self.note_practice.session = None;
            } else {
                self.restart_note_training();
            }
        }
        if self.note_practice.show_notes
            || self.note_practice.session.is_some()
            || self.training.is_some()
        {
            let index = if self.note_practice.session.is_some() {
                6
            } else {
                2
            };
            let (x, y) = control_pos(width, index, base);
            if nuon::button()
                .id("staff_zoom_out")
                .pos(x, y)
                .size(30.0, 30.0)
                .label("-")
                .border_radius([5.0; 4])
                .build(&mut self.nuon)
            {
                self.note_practice.zoom_percent =
                    self.note_practice.zoom_percent.saturating_sub(25).max(50);
            }
            nuon::label()
                .pos(x + 30.0, y)
                .size(70.0, 30.0)
                .text(format!("Staff {}%", self.note_practice.zoom_percent))
                .build(&mut self.nuon);
            if nuon::button()
                .id("staff_zoom_in")
                .pos(x + 100.0, y)
                .size(30.0, 30.0)
                .label("+")
                .border_radius([5.0; 4])
                .build(&mut self.nuon)
            {
                self.note_practice.zoom_percent = (self.note_practice.zoom_percent + 25).min(250);
            }
        }
        if self.note_practice.session.is_none() {
            return;
        }
        if toggle(
            &mut self.nuon,
            "note_range",
            control_pos(width, 2, base),
            self.note_practice.range.label().to_owned(),
            false,
        ) {
            self.note_practice.range = self.note_practice.range.next();
            self.restart_note_training();
        }
        if toggle(
            &mut self.nuon,
            "note_accidentals",
            control_pos(width, 3, base),
            format!(
                "Sharps: {}",
                if self.note_practice.chromatic {
                    "On"
                } else {
                    "Off"
                }
            ),
            self.note_practice.chromatic,
        ) {
            self.note_practice.chromatic = !self.note_practice.chromatic;
            self.restart_note_training();
        }
        if toggle(
            &mut self.nuon,
            "note_hint",
            control_pos(width, 4, base),
            format!(
                "Hint: {}",
                if self.note_practice.hint { "On" } else { "Off" }
            ),
            self.note_practice.hint,
        ) {
            self.note_practice.hint = !self.note_practice.hint;
        }
        if toggle(
            &mut self.nuon,
            "note_restart",
            control_pos(width, 5, base),
            "Restart notes".to_owned(),
            false,
        ) {
            self.restart_note_training();
        }
    }

    /// Queue notation before renderer preparation. Musical glyphs use their true
    /// baseline (not a label's bounding box) to align with lines and ledger lines.
    pub(super) fn update_note_panel(&mut self, ctx: &Context) {
        let practice = &self.note_practice;
        if !practice.show_notes && practice.session.is_none() && self.training.is_none() {
            return;
        }
        let top =
            self.note_controls_bottom(ctx) + if self.training.is_some() { 110.0 } else { 0.0 };
        let width = ctx.window_state.logical_size.width - 20.0;
        let available = self.keyboard.pos().y - top - if self.show_chords { 85.0 } else { 15.0 };
        if available < 155.0 || width < 280.0 {
            nuon::label()
                .pos(10.0, top)
                .size(width.max(0.0), 30.0)
                .text("Enlarge the window to see the note staves.")
                .build(&mut self.nuon);
            return;
        }
        let chord = self.training.as_ref().and_then(|s| s.exercise());
        let stacked = self.training.is_some();
        let requested_scale = f32::from(practice.zoom_percent) / 100.0;
        let mut scale = (available / 280.0).min(requested_scale);
        if stacked {
            scale = scale.min((width - 30.0) / 135.0);
        }
        // Enlarge notation without letting headings/feedback overwhelm the staff.
        let text_scale = scale.min(1.25);
        let target = practice.session.as_ref().map(|s| s.target());
        let correct = practice
            .session
            .as_ref()
            .is_some_and(|s| s.phase() == Phase::Correct)
            || self
                .training
                .as_ref()
                .is_some_and(|s| s.phase() == Phase::Correct);
        let mut heading = if let Some(s) = &practice.session {
            format!("Note practice - {}", s.target().clef.label())
        } else if let Some(exercise) = chord {
            format!("Chord notation - {}", exercise.symbol)
        } else {
            "Live notes - treble (G) and bass (F)".to_owned()
        };
        if scale + 0.01 < requested_scale {
            heading.push_str(&format!(
                " (fitted to {:.0}%; enlarge window for {}%)",
                scale * 100.0,
                practice.zoom_percent
            ));
        }
        let status = if let Some(s) = &practice.session {
            match s.phase() {
                Phase::ReleaseBeforeStart => "Release keys and sustain pedal to begin.".to_owned(),
                Phase::Correct => format!(
                    "Correct: {}! Release keys and pedal.",
                    pitch_label(target.unwrap().midi)
                ),
                Phase::Finished => "".to_owned(),
                Phase::Playing => {
                    if practice.hint {
                        format!(
                            "Find {}. Gold marker shows the key.",
                            pitch_label(target.unwrap().midi)
                        )
                    } else {
                        "Play the gold note in the correct octave. Hold for 0.3s.".to_owned()
                    }
                }
            }
        } else if chord.is_some() {
            "Gold: target chord. White: your played chord.".to_owned()
        } else {
            "White notes show the keys currently sounding. C4 = middle C.".to_owned()
        };
        quad(
            &mut self.quad_renderer_fg,
            10.0,
            top,
            width,
            280.0 * scale,
            [0.12, 0.12, 0.15, 1.0],
        );
        label(
            &mut self.nuon,
            15.0,
            top + 3.0 * scale,
            width - 10.0,
            22.0 * scale,
            17.0 * text_scale,
            heading,
        );
        label(
            &mut self.nuon,
            15.0,
            top + 26.0 * scale,
            width - 10.0,
            22.0 * scale,
            14.0 * text_scale,
            status,
        );

        let left = 25.0;
        for clef in [Clef::Treble, Clef::Bass] {
            let bottom = top + if clef == Clef::Treble { 115.0 } else { 215.0 } * scale;
            for i in 0..5 {
                quad(
                    &mut self.quad_renderer_fg,
                    left,
                    bottom - i as f32 * 10.0 * scale,
                    width - 30.0,
                    scale,
                    [0.65, 0.65, 0.7, 1.0],
                );
            }
            let (symbol, anchor) = match clef {
                Clef::Treble => (icons::sheet::g_clef(), bottom - 10.0 * scale),
                Clef::Bass => (icons::sheet::f_clef(), bottom - 30.0 * scale),
            };
            glyph(
                &mut self.text_renderer,
                left + 5.0 * scale,
                anchor,
                40.0 * scale,
                symbol,
                [225, 225, 235],
            );
        }
        if let Some(exercise) = chord {
            draw_chord(
                &mut self.quad_renderer_fg,
                &mut self.text_renderer,
                top,
                left + 90.0 * scale,
                exercise.notes,
                exercise.spelling,
                scale,
                if correct {
                    [80, 210, 120]
                } else {
                    [255, 195, 65]
                },
            );
        }
        if let Some(t) = target {
            draw_note(
                &mut self.quad_renderer_fg,
                &mut self.text_renderer,
                top,
                left + 80.0 * scale,
                t.midi,
                t.clef,
                scale,
                if correct {
                    [80, 210, 120]
                } else {
                    [255, 195, 65]
                },
            );
            if practice.hint || correct {
                self.note_key_marker(t.midi, correct);
            }
        }
        let live = self.live_notes.notes();
        // Account for enlarged noteheads, accidentals and spacing at higher zoom.
        let max_notes = if stacked {
            if width - 30.0 >= 245.0 * scale { 12 } else { 0 }
        } else {
            ((((width - 30.0) / scale - 163.0) / 28.0).floor().max(0.0) as usize).min(12)
        };
        let displayable: Vec<_> = live
            .iter()
            .copied()
            .filter(|n| (36..=84).contains(n))
            .collect();
        if stacked {
            let played: Vec<_> = displayable.iter().take(max_notes).copied().collect();
            draw_chord(
                &mut self.quad_renderer_fg,
                &mut self.text_renderer,
                top,
                left + 210.0 * scale,
                &played,
                chord.map(|e| e.spelling).unwrap_or_default(),
                scale,
                [225, 225, 235],
            );
        } else {
            for (i, n) in displayable.iter().take(max_notes).enumerate() {
                let x = left + 150.0 * scale + i as f32 * 28.0 * scale;
                draw_note(
                    &mut self.quad_renderer_fg,
                    &mut self.text_renderer,
                    top,
                    x,
                    *n,
                    if let Some(t) = target.filter(|t| t.midi == *n) {
                        t.clef
                    } else if *n < 60 {
                        Clef::Bass
                    } else {
                        Clef::Treble
                    },
                    scale,
                    [225, 225, 235],
                );
            }
        }
        let names = live
            .iter()
            .map(|n| pitch_label(*n))
            .collect::<Vec<_>>()
            .join(" ");
        let omitted = live.len() - displayable.len().min(max_notes);
        let footer = if omitted > 0 {
            format!(
                "Played: {}  ({} outside view/limit; notation C2-C6)",
                names, omitted
            )
        } else if live.is_empty() {
            "Played: -".to_owned()
        } else {
            format!("Played: {}", names)
        };
        label(
            &mut self.nuon,
            15.0,
            top + 254.0 * scale,
            width - 10.0,
            23.0 * scale,
            13.0 * text_scale,
            footer,
        );
    }

    fn note_key_marker(&mut self, midi: u8, correct: bool) {
        let layout = self.keyboard.layout();
        if !layout.range.contains(midi) {
            return;
        }
        let key = &layout.keys[(midi - layout.range.start()) as usize];
        quad(
            &mut self.quad_renderer_fg,
            key.x() + 2.0,
            self.keyboard.pos().y - 7.0,
            (key.width() - 4.0).max(1.0),
            5.0,
            if correct {
                [0.2, 0.8, 0.4, 1.0]
            } else {
                [1.0, 0.7, 0.1, 1.0]
            },
        );
    }
}

fn quad(out: &mut QuadRenderer, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
    out.push(QuadInstance {
        position: [x, y],
        size: [w, h],
        color,
        ..Default::default()
    });
}
fn label(ui: &mut nuon::Ui, x: f32, y: f32, w: f32, h: f32, size: f32, text: String) {
    nuon::label()
        .pos(x, y)
        .size(w, h)
        .font_size(size)
        .text(text)
        .text_justify(nuon::TextAlign::Center)
        .build(ui);
}
fn glyph(out: &mut TextRenderer, x: f32, baseline: f32, size: f32, text: &str, color: [u8; 3]) {
    let buffer = TextRenderer::gen_buffer_with_attr(
        size,
        text,
        cosmic_text::Attrs::new()
            .family(cosmic_text::Family::Name("Leland"))
            .color(cosmic_text::Color::rgb(color[0], color[1], color[2])),
    );
    let line_y = buffer.layout_runs().next().map(|r| r.line_y).unwrap_or(0.0);
    out.queue_buffer(x, baseline - line_y, buffer);
}
#[allow(clippy::too_many_arguments)]
fn draw_note(
    quads: &mut QuadRenderer,
    text: &mut TextRenderer,
    top: f32,
    x: f32,
    midi: u8,
    clef: Clef,
    scale: f32,
    color: [u8; 3],
) {
    let p = staff_pitch(midi, clef);
    let bottom = top + if clef == Clef::Treble { 115.0 } else { 215.0 } * scale;
    let y = bottom - p.step as f32 * 5.0 * scale;
    let rgba = [
        color[0] as f32 / 255.0,
        color[1] as f32 / 255.0,
        color[2] as f32 / 255.0,
        1.0,
    ];
    for step in ledger_steps(p.step) {
        quad(
            quads,
            x - 4.0 * scale,
            bottom - step as f32 * 5.0 * scale,
            21.0 * scale,
            scale,
            [0.7, 0.7, 0.75, 1.0],
        );
    }
    if p.sharp {
        glyph(
            text,
            x - 13.0 * scale,
            y,
            40.0 * scale,
            icons::sheet::accidental_sharp(),
            color,
        );
    }
    glyph(
        text,
        x,
        y,
        40.0 * scale,
        icons::sheet::notehead_black(),
        color,
    );
    if p.step < 4 {
        quad(
            quads,
            x + 12.0 * scale,
            y - 30.0 * scale,
            scale,
            30.0 * scale,
            rgba,
        );
    } else {
        quad(quads, x, y, scale, 30.0 * scale, rgba);
    }
}

/// Chord-only stack with one shared stem per staff. Single-note drawing above
/// remains unchanged; seconds move to the other side of the common stem.
#[allow(clippy::too_many_arguments)]
fn draw_chord(
    quads: &mut QuadRenderer,
    text: &mut TextRenderer,
    top: f32,
    x: f32,
    notes: &[u8],
    spelling: Spelling,
    scale: f32,
    color: [u8; 3],
) {
    let layout = chord_staff_notes(notes, spelling);
    let rgba = [
        color[0] as f32 / 255.0,
        color[1] as f32 / 255.0,
        color[2] as f32 / 255.0,
        1.0,
    ];
    for clef in [Clef::Treble, Clef::Bass] {
        let group: Vec<_> = layout.iter().filter(|n| n.clef == clef).collect();
        let (Some(first), Some(last)) = (group.first(), group.last()) else {
            continue;
        };
        let up = first.step + last.step < 8;
        let bottom = top + if clef == Clef::Treble { 115.0 } else { 215.0 } * scale;
        let left_shift = if !up && group.iter().any(|n| n.head_column > 0) {
            13.0
        } else {
            0.0
        };
        for note in &group {
            let head_x =
                x + f32::from(note.head_column) * 13.0 * scale * if up { 1.0 } else { -1.0 };
            let y = bottom - note.step as f32 * 5.0 * scale;
            for step in ledger_steps(note.step) {
                quad(
                    quads,
                    head_x - 4.0 * scale,
                    bottom - step as f32 * 5.0 * scale,
                    21.0 * scale,
                    scale,
                    [0.7, 0.7, 0.75, 1.0],
                );
            }
            if let Some(accidental) = note.accidental {
                glyph(
                    text,
                    x - (left_shift + 13.0 + note.accidental_column as f32 * 14.0) * scale,
                    y,
                    40.0 * scale,
                    if accidental == Spelling::Flats {
                        icons::sheet::accidental_flat()
                    } else {
                        icons::sheet::accidental_sharp()
                    },
                    color,
                );
            }
            glyph(
                text,
                head_x,
                y,
                40.0 * scale,
                icons::sheet::notehead_black(),
                color,
            );
        }
        let high_y = bottom - last.step as f32 * 5.0 * scale;
        let low_y = bottom - first.step as f32 * 5.0 * scale;
        quad(
            quads,
            if up { x + 12.0 * scale } else { x },
            if up { high_y - 30.0 * scale } else { high_y },
            scale,
            low_y - high_y + 30.0 * scale,
            rgba,
        );
    }
}
