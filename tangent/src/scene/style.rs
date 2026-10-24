use tangent_core::{
    config::{Palette, VisualSettings},
    render::{QuadInstance, QuadRenderer},
};

pub(crate) fn apply(ui: &mut nuon::Ui, settings: VisualSettings) {
    let p = settings.theme.palette();
    ui.theme = nuon::Theme {
        surface: p.surface.into(),
        button: p.button.into(),
        hover: nuon::Color::from(p.button).mix(p.accent, 0.16),
        selected: nuon::Color::from(p.button).mix(p.accent, 0.28),
        accent: p.accent.into(),
        muted: p.muted.into(),
        border: Palette::rgba(p.staff, 0.16).into(),
    };
}

/// A translucent notation surface; noteheads and staff lines stay crisp/opaque.
pub(crate) fn staff_panel(
    out: &mut QuadRenderer,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    settings: VisualSettings,
) {
    let p = settings.theme.palette();
    for (offset, color, border_width) in [
        (4.0, [0.0, 0.0, 0.0, 0.16], 0.0),
        (0.0, Palette::linear(p.surface, settings.panel_alpha()), 1.0),
    ] {
        out.push(QuadInstance {
            position: [x, y + offset],
            size: [w, h],
            color,
            border_radius: [12.0; 4],
            border_color: Palette::linear(p.staff, 0.2),
            border_width,
        });
    }
}

/// Used in both the main Settings page and the in-song settings drawer.
pub(crate) fn settings(ctx: &mut crate::context::Context, ui: &mut nuon::Ui, width: f32) {
    let original = ctx.config.visuals();
    let mut v = original;
    nuon::settings_section("Appearance")
        .width(width)
        .build(ui, |ui, rows, spacer| {
            nuon::settings_row()
                .title("Theme")
                .subtitle("Color palette")
                .body(|ui, w, h| {
                    if nuon::button()
                        .id("appearance_theme")
                        .pos(w - 125.0, nuon::center_y(h, 32.0))
                        .size(125.0, 32.0)
                        .label(v.theme.name())
                        .build(ui)
                    {
                        v.theme = v.theme.next();
                    }
                })
                .build(ui, rows);
            spacer(ui);
            if nuon::settings_row_toggler()
                .id("appearance_gradient")
                .title("Soft gradients")
                .subtitle("A quiet, static background")
                .value(v.gradients)
                .build(ui, rows)
            {
                v.gradients = !v.gradients;
            }
        });
    nuon::settings_section("Piano colors")
        .width(width)
        .build(ui, |ui, rows, _| {
            if nuon::settings_row_toggler()
                .id("appearance_soft_notes")
                .title("Soft note colors")
                .subtitle("Use the theme palette for piano and waterfall")
                .value(v.soft_note_colors)
                .build(ui, rows)
            {
                v.soft_note_colors = !v.soft_note_colors;
            }
        });
    nuon::settings_section("Notation")
        .width(width)
        .build(ui, |ui, rows, spacer| {
            let change = nuon::settings_row_spin()
                .title("Staff size")
                .subtitle(format!("{}% · fits available space", v.staff_size))
                .id("appearance_staff_size")
                .build(ui, rows);
            match change {
                nuon::SettingsRowSpinResult::Plus => v.staff_size = (v.staff_size + 25).min(250),
                nuon::SettingsRowSpinResult::Minus => {
                    v.staff_size = v.staff_size.saturating_sub(25).max(50)
                }
                _ => {}
            }
            spacer(ui);
            let change = nuon::settings_row_spin()
                .title("Staff background")
                .subtitle(if v.theme == tangent_core::config::Theme::HighContrast {
                    "Opaque in High Contrast".to_owned()
                } else {
                    format!("{}% opacity", v.staff_opacity)
                })
                .id("appearance_opacity")
                .build(ui, rows);
            match change {
                nuon::SettingsRowSpinResult::Plus => {
                    v.staff_opacity = (v.staff_opacity + 5).min(100)
                }
                nuon::SettingsRowSpinResult::Minus => {
                    v.staff_opacity = v.staff_opacity.saturating_sub(5).max(55)
                }
                _ => {}
            }
            spacer(ui);
            if nuon::settings_row_toggler()
                .id("appearance_score")
                .title("MIDI staff")
                .subtitle("Show notation during playback")
                .value(v.score_enabled)
                .build(ui, rows)
            {
                v.score_enabled = !v.score_enabled;
            }
            spacer(ui);
            if nuon::settings_row_toggler()
                .id("appearance_scroll")
                .title("Scrolling notation")
                .subtitle("Off: stationary pages")
                .value(v.score_scroll)
                .build(ui, rows)
            {
                v.score_scroll = !v.score_scroll;
            }
            spacer(ui);
            nuon::settings_row()
                .title("Visible beats")
                .subtitle("MIDI staff time window")
                .body(|ui, w, h| {
                    if nuon::button()
                        .id("appearance_beats")
                        .pos(w - 70.0, nuon::center_y(h, 32.0))
                        .size(70.0, 32.0)
                        .label(v.score_beats.to_string())
                        .build(ui)
                    {
                        v.score_beats = match v.score_beats {
                            4 => 8,
                            8 => 16,
                            _ => 4,
                        };
                    }
                })
                .build(ui, rows);
            spacer(ui);
            if nuon::settings_row_toggler()
                .id("appearance_flats")
                .title("Prefer flats")
                .subtitle("MIDI staff: ♭ instead of ♯")
                .value(v.flats)
                .build(ui, rows)
            {
                v.flats = !v.flats;
            }
            spacer(ui);
            if nuon::settings_row_toggler()
                .id("appearance_feedback")
                .title("Your note marker")
                .subtitle("Played pitches and semitone distance")
                .value(v.input_feedback)
                .build(ui, rows)
            {
                v.input_feedback = !v.input_feedback;
            }
            spacer(ui);
            if nuon::settings_row_toggler()
                .id("appearance_rails")
                .title("Duration guides")
                .subtitle("Show exact MIDI note lengths")
                .value(v.duration_rails)
                .build(ui, rows)
            {
                v.duration_rails = !v.duration_rails;
            }
            spacer(ui);
            if nuon::settings_row_toggler()
                .id("appearance_grid")
                .title("Beat grid")
                .subtitle("Quarter-note timing guides")
                .value(v.beat_grid)
                .build(ui, rows)
            {
                v.beat_grid = !v.beat_grid;
            }
        });
    nuon::settings_section("Practice")
        .width(width)
        .build(ui, |ui, rows, _| {
            if nuon::settings_row_toggler()
                .id("appearance_hear")
                .title("Play exercise notes")
                .subtitle("Hear note and chord tasks on output")
                .value(v.practice_sound)
                .build(ui, rows)
            {
                v.practice_sound = !v.practice_sound;
            }
        });
    if v != original {
        ctx.config.set_visuals(v);
        ctx.config.save();
        apply(ui, v);
    }
}

pub(crate) fn drawer_covers_pointer(ctx: &crate::context::Context, top: f32) -> bool {
    let width = (ctx.window_state.logical_size.width - 16.0).clamp(280.0, 420.0);
    ctx.window_state.cursor_logical_position.x >= ctx.window_state.logical_size.width - width
        && ctx.window_state.cursor_logical_position.y >= top
}

pub(crate) fn drawer(ctx: &mut crate::context::Context, ui: &mut nuon::Ui, top: f32) {
    let width = (ctx.window_state.logical_size.width - 16.0).clamp(280.0, 420.0);
    let height = (ctx.window_state.logical_size.height - top - 12.0).max(0.0);
    nuon::translate()
        .pos(ctx.window_state.logical_size.width - width, top)
        .build(ui, |ui| {
            nuon::layer().overlay(true).build(ui, |ui| {
                nuon::quad()
                    .size(width, height)
                    .color(ui.theme.surface)
                    .border_color(ui.theme.border)
                    .border_width(1.0)
                    .border_radius([12.0, 0.0, 0.0, 12.0])
                    .build(ui);
                nuon::scroll("freeplay_settings_scroll")
                    .scissor_size(width, height)
                    .build(ui, |ui| {
                        nuon::translate().x(12.0).add_to_current(ui);
                        settings(ctx, ui, width - 24.0);
                    });
                nuon::click_area("freeplay_settings_drawer")
                    .size(width, height)
                    .build(ui);
            });
        });
}
