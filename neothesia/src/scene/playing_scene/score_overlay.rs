use std::collections::BTreeMap;

use crate::{
    context::Context,
    scene::notation::{MusicGlyphs, label, quad},
    song::Song,
};
use midi_file::tempo_track::TempoTrack;
use neothesia_core::{
    chords::{
        Spelling,
        notes::{Clef, ledger_steps, staff_pitch},
    },
    render::{QuadRenderer, TextRenderer},
    score::{NoteValue, Rhythm, Score, ScoreNote, ScoreWindow, ViewMode},
};

pub(super) struct ScoreOverlay {
    enabled: bool,
    mode: ViewMode,
    span: f64,
    zoom: u16,
    spelling: Spelling,
    score: Score,
    upper: f32,
    lower: f32,
    glyphs: MusicGlyphs,
}
impl ScoreOverlay {
    pub fn new(song: &Song) -> Self {
        let notes = song
            .file
            .tracks
            .iter()
            .filter(|t| {
                song.config
                    .tracks
                    .get(t.track_id)
                    .is_some_and(|t| t.visible)
            })
            .flat_map(|t| t.notes.iter())
            .filter(|n| n.channel != 9 && n.velocity > 0)
            .map(|n| ScoreNote {
                start: song
                    .file
                    .tempo_track
                    .seconds_to_quarters(n.start.as_secs_f64()),
                end: song
                    .file
                    .tempo_track
                    .seconds_to_quarters(n.end.as_secs_f64()),
                pitch: n.note,
                track: n.track_id,
            })
            .collect();
        let score = Score::new(notes);
        let upper = score
            .notes()
            .iter()
            .filter(|n| n.pitch >= 60)
            .map(|n| staff_pitch(n.pitch, Clef::Treble).step + 1)
            .max()
            .unwrap_or(8)
            .max(8)
            - 8;
        let lower = -score
            .notes()
            .iter()
            .filter(|n| n.pitch < 60)
            .map(|n| staff_pitch(n.pitch, Clef::Bass).step)
            .min()
            .unwrap_or(0)
            .min(0);
        Self {
            enabled: false,
            mode: ViewMode::Scrolling,
            span: 8.0,
            zoom: 125,
            spelling: Spelling::Sharps,
            score,
            upper: upper as f32 * 5.0,
            lower: lower as f32 * 5.0,
            glyphs: MusicGlyphs::default(),
        }
    }

    fn control_pos(width: f32, index: usize) -> (f32, f32) {
        let columns = ((width - 20.0) / 135.0).floor().max(1.0) as usize;
        (
            10.0 + (index % columns) as f32 * 135.0,
            85.0 + (index / columns) as f32 * 35.0,
        )
    }
    fn button(ui: &mut nuon::Ui, width: f32, index: usize, id: &'static str, text: &str) -> bool {
        let (x, y) = Self::control_pos(width, index);
        nuon::button()
            .id(id)
            .pos(x, y)
            .size(130.0, 30.0)
            .label(text.to_owned())
            .color([37, 35, 42, 245])
            .border_radius([5.0; 4])
            .build(ui)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        ctx: &Context,
        seconds: f64,
        tempo: &TempoTrack,
        keyboard_y: f32,
        ui: &mut nuon::Ui,
        quads: &mut QuadRenderer,
        text: &mut TextRenderer,
    ) {
        let width = ctx.window_state.logical_size.width;
        if Self::button(
            ui,
            width,
            0,
            "playback_score_toggle",
            if self.enabled {
                "Score: On"
            } else {
                "Score: Off"
            },
        ) {
            self.enabled = !self.enabled;
        }
        if !self.enabled {
            return;
        }
        if Self::button(
            ui,
            width,
            1,
            "playback_score_mode",
            match self.mode {
                ViewMode::Paged => "View: Pages",
                ViewMode::Scrolling => "View: Scroll",
            },
        ) {
            self.mode = match self.mode {
                ViewMode::Paged => ViewMode::Scrolling,
                ViewMode::Scrolling => ViewMode::Paged,
            };
        }
        if Self::button(
            ui,
            width,
            2,
            "playback_score_span",
            &format!("Window: {} beats", self.span),
        ) {
            self.span = if self.span == 4.0 {
                8.0
            } else if self.span == 8.0 {
                16.0
            } else {
                4.0
            };
        }
        let (x, y) = Self::control_pos(width, 3);
        if nuon::button()
            .id("playback_score_zoom_out")
            .pos(x, y)
            .size(30.0, 30.0)
            .label("-")
            .build(ui)
        {
            self.zoom = self.zoom.saturating_sub(25).max(50);
        }
        label(
            ui,
            x + 30.0,
            y,
            70.0,
            30.0,
            13.0,
            format!("Staff {}%", self.zoom),
        );
        if nuon::button()
            .id("playback_score_zoom_in")
            .pos(x + 100.0, y)
            .size(30.0, 30.0)
            .label("+")
            .build(ui)
        {
            self.zoom = (self.zoom + 25).min(250);
        }
        if Self::button(
            ui,
            width,
            4,
            "playback_score_spelling",
            if self.spelling == Spelling::Sharps {
                "Spelling: #"
            } else {
                "Spelling: b"
            },
        ) {
            self.spelling = if self.spelling == Spelling::Sharps {
                Spelling::Flats
            } else {
                Spelling::Sharps
            };
        }
        let top = Self::control_pos(width, 4).1 + 40.0;
        let available = keyboard_y - top - 45.0;
        if available < 155.0 || width < 300.0 {
            label(
                ui,
                10.0,
                top,
                (width - 20.0).max(0.0),
                30.0,
                14.0,
                "Enlarge the window to see the MIDI score.".to_owned(),
            );
            return;
        }
        let base_height = 280.0 + self.upper + self.lower;
        let requested = f32::from(self.zoom) / 100.0;
        let scale = (available / base_height)
            .min(requested)
            .min((width - 30.0) / 180.0);
        let panel_width = width - 20.0;
        quad(
            quads,
            10.0,
            top,
            panel_width,
            base_height * scale,
            [0.1, 0.11, 0.14, 1.0],
        );
        let beat = tempo.seconds_to_quarters(seconds);
        let window = ScoreWindow::new(beat, self.span, self.mode);
        let left = 25.0 + 65.0 * scale;
        let right = width - 25.0 - 40.0 * scale;
        let plot_width = right - left;
        let visible: Vec<_> = self.score.visible(window).take(513).collect();
        let density = if visible.len() > 512 {
            " · dense: reduce Window"
        } else {
            ""
        };
        let fit = if scale + 0.01 < requested {
            format!(" · fit {:.0}%", scale * 100.0)
        } else {
            String::new()
        };
        label(
            ui,
            15.0,
            top + 3.0 * scale,
            panel_width - 10.0,
            23.0 * scale,
            14.0 * scale.min(1.25),
            format!(
                "MIDI staff · beat {:.1} · {}{}{}",
                beat.max(0.0) + 1.0,
                if self.score.notes().is_empty() {
                    "no visible pitched notes"
                } else {
                    "rhythm approximated"
                },
                fit,
                density
            ),
        );
        let staff_bottom =
            |clef| top + (if clef == Clef::Treble { 115.0 } else { 215.0 } + self.upper) * scale;
        let staff_top = staff_bottom(Clef::Treble) - 65.0 * scale;
        let staff_end = staff_bottom(Clef::Bass) + self.lower * scale + 12.0 * scale;
        for b in window.start.ceil() as i64..=window.end.floor() as i64 {
            if b < 0 {
                continue;
            }
            let x = left + window.fraction(b as f64) as f32 * plot_width;
            quad(
                quads,
                x,
                staff_top,
                0.75,
                staff_end - staff_top,
                [0.2, 0.22, 0.27, 1.0],
            );
            label(
                ui,
                x - 18.0,
                top + 27.0 * scale,
                36.0,
                18.0 * scale,
                11.0 * scale.min(1.25),
                format!("{}", b + 1),
            );
        }
        for clef in [Clef::Treble, Clef::Bass] {
            let bottom = staff_bottom(clef);
            for i in 0..5 {
                quad(
                    quads,
                    25.0,
                    bottom - i as f32 * 10.0 * scale,
                    width - 50.0,
                    scale,
                    [0.6, 0.63, 0.7, 1.0],
                );
            }
            self.glyphs.draw(
                text,
                30.0,
                bottom - if clef == Clef::Treble { 10.0 } else { 30.0 } * scale,
                40.0 * scale,
                if clef == Clef::Treble {
                    '\u{E050}'
                } else {
                    '\u{E062}'
                },
                [215, 220, 235],
            );
        }
        // Same-onset/same-written-duration tones share a stem on each staff.
        let mut groups: BTreeMap<(i64, u8, u8, bool), Vec<&ScoreNote>> = BTreeMap::new();
        for note in visible.iter().take(512) {
            let rhythm = Rhythm::nearest(note.end - note.start);
            groups
                .entry((
                    (note.start * 1_000_000.0).round() as i64,
                    u8::from(note.pitch >= 60),
                    rhythm.value as u8,
                    rhythm.dotted,
                ))
                .or_default()
                .push(note);
            let clef = if note.pitch < 60 {
                Clef::Bass
            } else {
                Clef::Treble
            };
            let pitch = staff_pitch(note.pitch, clef);
            let step = pitch.step
                + if pitch.sharp && self.spelling == Spelling::Flats {
                    1
                } else {
                    0
                };
            let y = staff_bottom(clef) - step as f32 * 5.0 * scale;
            let x1 = left + window.fraction(note.start.max(window.start)) as f32 * plot_width;
            let x2 = left + window.fraction(note.end.min(window.end)) as f32 * plot_width;
            if x2 > x1 {
                quad(
                    quads,
                    x1,
                    y + 7.0 * scale,
                    x2 - x1,
                    scale,
                    if note.active(beat) {
                        [0.2, 0.65, 0.5, 1.0]
                    } else {
                        [0.22, 0.27, 0.35, 1.0]
                    },
                );
            }
        }
        for group in groups.values() {
            let note = group[0];
            let x = left + window.fraction(note.start.max(window.start)) as f32 * plot_width;
            draw_group(
                &mut self.glyphs,
                quads,
                text,
                group,
                x,
                staff_bottom(if note.pitch < 60 {
                    Clef::Bass
                } else {
                    Clef::Treble
                }),
                scale,
                beat,
                self.spelling,
            );
        }
        let cursor = left + window.cursor as f32 * plot_width;
        quad(
            quads,
            cursor,
            staff_top,
            2.0,
            staff_end - staff_top,
            [0.3, 0.75, 1.0, 1.0],
        );
        let carried = visible.iter().any(|n| n.start < window.start);
        label(
            ui,
            15.0,
            top + (base_height - 23.0) * scale,
            panel_width - 10.0,
            20.0 * scale,
            12.0 * scale.min(1.25),
            if seconds < 0.0 {
                format!("Starts in {:.1}s", -seconds)
            } else if carried {
                "Held notes from before this window are shown at its left edge.".to_owned()
            } else {
                "Quarter-note beat grid · green = current notes · thin rails = exact MIDI duration"
                    .to_owned()
            },
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_group(
    glyphs: &mut MusicGlyphs,
    quads: &mut QuadRenderer,
    text: &mut TextRenderer,
    notes: &[&ScoreNote],
    x: f32,
    bottom: f32,
    scale: f32,
    beat: f64,
    spelling: Spelling,
) {
    let clef = if notes[0].pitch < 60 {
        Clef::Bass
    } else {
        Clef::Treble
    };
    let rhythm = Rhythm::nearest(notes[0].end - notes[0].start);
    let mut pitches: Vec<_> = notes.iter().map(|n| n.pitch).collect();
    pitches.sort_unstable();
    pitches.dedup();
    let positions: Vec<_> = pitches
        .iter()
        .map(|p| {
            let n = staff_pitch(*p, clef);
            (
                n.step
                    + if n.sharp && spelling == Spelling::Flats {
                        1
                    } else {
                        0
                    },
                n.sharp,
            )
        })
        .collect();
    let up = positions.first().unwrap().0 + positions.last().unwrap().0 < 8;
    let head_width = if rhythm.value == NoteValue::Whole {
        18.0
    } else {
        13.0
    };
    let mut last_step = None;
    let mut last_column = false;
    let mut accidental_columns: Vec<(i16, usize)> = Vec::new();
    let mut heads = Vec::new();
    for (step, sharp) in &positions {
        let column = last_step.is_some_and(|s| step - s <= 1) && !last_column;
        heads.push((*step, *sharp, column));
        last_step = Some(*step);
        last_column = column;
    }
    let left_shift = if !up && heads.iter().any(|n| n.2) {
        head_width
    } else {
        0.0
    };
    let color = if notes.iter().any(|n| n.active(beat)) {
        [80, 225, 160]
    } else if notes.iter().all(|n| n.end <= beat) {
        [135, 145, 165]
    } else {
        [230, 232, 240]
    };
    for (step, sharp, column) in &heads {
        let hx = x + if *column {
            head_width * scale * if up { 1.0 } else { -1.0 }
        } else {
            0.0
        };
        let y = bottom - *step as f32 * 5.0 * scale;
        for ledger in ledger_steps(*step) {
            quad(
                quads,
                hx - 4.0 * scale,
                bottom - ledger as f32 * 5.0 * scale,
                (head_width + 8.0) * scale,
                scale,
                [0.6, 0.63, 0.7, 1.0],
            );
        }
        if *sharp {
            let mut column = 0;
            while accidental_columns
                .iter()
                .any(|(s, c)| *c == column && (step - s).abs() < 7)
            {
                column += 1;
            }
            accidental_columns.push((*step, column));
            glyphs.draw(
                text,
                x - (left_shift + 13.0 + column as f32 * 14.0) * scale,
                y,
                40.0 * scale,
                if spelling == Spelling::Flats {
                    '\u{E260}'
                } else {
                    '\u{E262}'
                },
                color,
            );
        }
        glyphs.draw(
            text,
            hx,
            y,
            40.0 * scale,
            match rhythm.value {
                NoteValue::Whole => '\u{E0A2}',
                NoteValue::Half => '\u{E0A3}',
                _ => '\u{E0A4}',
            },
            color,
        );
        if rhythm.dotted {
            let dot_y = y - if step % 2 == 0 { 5.0 * scale } else { 0.0 };
            draw_dot(quads, hx + (head_width + 5.0) * scale, dot_y, scale, color);
        }
    }
    if rhythm.value != NoteValue::Whole {
        let high = bottom - heads.last().unwrap().0 as f32 * 5.0 * scale;
        let low = bottom - heads.first().unwrap().0 as f32 * 5.0 * scale;
        let stem_x = if up { x + 12.0 * scale } else { x };
        let tip = if up {
            high - 30.0 * scale
        } else {
            low + 30.0 * scale
        };
        quad(
            quads,
            stem_x,
            if up { tip } else { high },
            scale,
            low - high + 30.0 * scale,
            [
                color[0] as f32 / 255.0,
                color[1] as f32 / 255.0,
                color[2] as f32 / 255.0,
                1.0,
            ],
        );
        if rhythm.flags() > 0 {
            let code = 0xE240 + 2 * u32::from(rhythm.flags() - 1) + u32::from(!up);
            glyphs.draw(
                text,
                stem_x,
                tip,
                40.0 * scale,
                char::from_u32(code).unwrap(),
                color,
            );
        }
    }
}
fn draw_dot(out: &mut QuadRenderer, x: f32, y: f32, scale: f32, color: [u8; 3]) {
    out.push(neothesia_core::render::QuadInstance {
        position: [x, y - 1.5 * scale],
        size: [3.0 * scale; 2],
        border_radius: [1.5 * scale; 4],
        color: [
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
            1.0,
        ],
        ..Default::default()
    });
}
