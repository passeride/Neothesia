use std::time::Duration;

use midi_file::midly::MidiMessage;
use tangent_core::chords::{self, RecognitionOptions, live::LiveNotes};
use tangent_core::render::{GlowRenderer, GuidelineRenderer, QuadRenderer, TextRenderer};
use winit::{
    event::WindowEvent,
    keyboard::{Key, NamedKey},
};

use crate::{
    TangentEvent,
    context::Context,
    scene::{
        MouseToMidiEventState, NuonRenderer, Scene,
        freeplay::recorder::{FreeplayRecorder, Preview, RecorderStatus},
        playing_scene::Keyboard,
    },
    song::Song,
    utils::{BoxFuture, noop_waker_ref, window::WinitEvent},
};

mod notes;
mod practice_audio;
mod recorder;
mod training;

type MsgFn = Box<dyn FnOnce(&mut FreeplayScene, &mut Context)>;

fn on_async<T, Fut, FN>(future: Fut, f: FN) -> BoxFuture<MsgFn>
where
    T: 'static,
    Fut: Future<Output = T> + Send + 'static,
    FN: FnOnce(T, &mut FreeplayScene, &mut Context) + Send + 'static,
{
    Box::pin(async {
        let res = future.await;
        let f: MsgFn = Box::new(move |state, ctx| f(res, state, ctx));
        f
    })
}

pub struct FreeplayScene {
    settings_open: bool,
    background: tangent_core::render::BgPipeline,
    keyboard: Keyboard,
    guidelines: GuidelineRenderer,

    text_renderer: TextRenderer,
    quad_renderer_bg: QuadRenderer,
    quad_renderer_fg: QuadRenderer,
    glow: Option<GlowRenderer>,

    // TODO: This does not make sens, but get's us going without refactoring
    song: Option<Song>,

    nuon_renderer: NuonRenderer,
    nuon: nuon::Ui,
    mouse_to_midi_state: MouseToMidiEventState,
    show_chords: bool,
    practice_audio: practice_audio::PracticeAudio,
    note_practice: notes::NotePractice,
    training: Option<chords::training::TrainingSession>,
    live_notes: LiveNotes,
    chord_text: String,

    recorder: FreeplayRecorder,
    recorder_status: RecorderStatus,
    preview: Option<Preview>,

    context: std::task::Context<'static>,
    futures: Vec<BoxFuture<MsgFn>>,
}

impl FreeplayScene {
    pub fn new(ctx: &mut Context, song: Option<Song>) -> Self {
        let mut keyboard = Keyboard::new(ctx, Default::default());
        keyboard.set_pressed_by_user_colors(ctx.config.color_schema()[0].clone());

        let keyboard_layout = keyboard.layout();

        let guidelines = GuidelineRenderer::new(
            keyboard_layout.clone(),
            *keyboard.pos(),
            ctx.config.vertical_guidelines(),
            false,
            Default::default(),
        );

        let text_renderer = ctx.text_renderer_factory.new_renderer();

        let quad_renderer_bg = ctx.quad_renderer_factory.new_renderer();
        let quad_renderer_fg = ctx.quad_renderer_factory.new_renderer();

        let glow = ctx.config.glow().then_some(GlowRenderer::new(
            &ctx.gpu,
            &ctx.transform,
            keyboard.layout(),
        ));

        Self {
            settings_open: false,
            background: tangent_core::render::BgPipeline::new(&ctx.gpu),
            keyboard,
            guidelines,
            text_renderer,
            quad_renderer_bg,
            quad_renderer_fg,
            glow,
            song,
            nuon_renderer: NuonRenderer::new(ctx),
            nuon: nuon::Ui::new(),
            mouse_to_midi_state: MouseToMidiEventState::default(),
            show_chords: false,
            practice_audio: practice_audio::PracticeAudio::new(ctx.config.visuals().practice_sound),
            note_practice: notes::NotePractice::default(),
            training: None,
            live_notes: LiveNotes::default(),
            chord_text: String::new(),
            recorder: FreeplayRecorder::default(),
            recorder_status: RecorderStatus::default(),
            preview: None,

            context: std::task::Context::from_waker(noop_waker_ref()),
            futures: Vec::new(),
        }
    }

    fn update_glow(&mut self, delta: Duration) {
        let Some(glow) = &mut self.glow else {
            return;
        };

        glow.clear();

        let keys = &self.keyboard.layout().keys;
        let states = self.keyboard.key_states();

        for (key, state) in keys.iter().zip(states) {
            let Some(mut color) = state.pressed_by_user().copied() else {
                continue;
            };

            color.r *= 0.5;
            color.g *= 0.5;
            color.b *= 0.5;

            glow.push(
                key.id(),
                color,
                key.x(),
                self.keyboard.pos().y,
                key.width(),
                delta,
            );
        }
    }

    fn update_ui(&mut self, ctx: &mut Context) {
        recorder::update_preview_ui(self, ctx);

        if nuon::button()
            .id("freeplay_show_chords")
            .pos(10.0, if self.preview.is_some() { 85.0 } else { 40.0 })
            .size(130.0, 30.0)
            .label(if self.show_chords {
                "Chords: On"
            } else {
                "Chords: Off"
            })
            .color(if self.show_chords {
                self.nuon.theme.selected
            } else {
                self.nuon.theme.button
            })
            .border_radius([8.0; 4])
            .build(&mut self.nuon)
        {
            self.show_chords = !self.show_chords;
            self.update_chord_text();
        }
        self.update_training_ui(ctx);
        self.update_note_controls(ctx);
        self.update_practice_audio_controls(ctx);
        if !self.show_chords {
            return;
        }

        nuon::label()
            .text(&self.chord_text)
            .font_size(22.0)
            .y(self.keyboard.pos().y - 80.0)
            .height(70.0)
            .text_justify(nuon::TextAlign::Center)
            .width(ctx.window_state.logical_size.width)
            .build(&mut self.nuon);
    }

    fn update_chord_text(&mut self) {
        if !self.show_chords {
            return;
        }
        let options = RecognitionOptions::default();
        let analysis = chords::recognize(&self.live_notes.notes(), options);
        self.chord_text = if analysis.notes.is_empty() {
            "Play a chord".to_owned()
        } else if analysis.pitch_classes.len() == 1 {
            format!(
                "Note: {}",
                chords::note_name(analysis.pitch_classes[0], options.spelling)
            )
        } else if let Some(best) = analysis.candidates.first() {
            let alternatives: Vec<_> = analysis
                .candidates
                .iter()
                .skip(1)
                .take(2)
                .map(|c| c.symbol.as_str())
                .collect();
            if alternatives.is_empty() {
                best.symbol.clone()
            } else {
                format!("{}\nAlso: {}", best.symbol, alternatives.join(" | "))
            }
        } else {
            let names: Vec<_> = analysis
                .pitch_classes
                .iter()
                .map(|n| chords::note_name(*n, options.spelling))
                .collect();
            format!("No chord match: {}", names.join(" "))
        };
    }

    fn resize(&mut self, ctx: &mut Context) {
        self.keyboard.resize(ctx);
        self.guidelines.set_layout(self.keyboard.layout().clone());
        self.guidelines.set_pos(*self.keyboard.pos());
        if let Some(preview) = self.preview.as_mut() {
            preview.resize(&self.keyboard, ctx);
        }
    }

    fn dispatch_futures(&mut self, ctx: &mut Context) {
        let mut cbs = Vec::new();
        self.futures
            .retain_mut(|f| match f.as_mut().poll(&mut self.context) {
                std::task::Poll::Ready(msg) => {
                    cbs.push(msg);
                    false
                }
                std::task::Poll::Pending => true,
            });

        for cb in cbs {
            cb(self, ctx);
        }
    }
}

impl Scene for FreeplayScene {
    fn update(&mut self, ctx: &mut Context, delta: Duration) {
        super::style::apply(&mut self.nuon, ctx.config.visuals());
        self.background.update(ctx.config.visuals());
        self.quad_renderer_bg.clear();
        self.quad_renderer_fg.clear();

        self.dispatch_futures(ctx);
        if let Some(session) = &mut self.note_practice.session {
            session.update(&self.live_notes.notes(), delta);
        }
        if let Some(session) = &mut self.training {
            session.update(&self.live_notes.notes(), delta);
        }

        self.update_practice_audio(ctx, delta);

        if let Some(preview) = self.preview.as_mut() {
            preview.update(&mut self.keyboard, ctx, delta);
        }

        let time = 0.0;

        self.guidelines.update(
            &mut self.quad_renderer_bg,
            ctx.config.animation_speed(),
            ctx.window_state.scale_factor as f32,
            time,
            ctx.window_state.logical_size,
        );
        self.keyboard
            .update(&mut self.quad_renderer_fg, &mut self.text_renderer);

        self.update_note_panel(ctx);
        self.update_glow(delta);

        self.quad_renderer_bg.prepare();
        self.quad_renderer_fg.prepare();

        if let Some(glow) = &mut self.glow {
            glow.prepare();
        }

        self.text_renderer.update(
            ctx.window_state.physical_size,
            ctx.window_state.scale_factor as f32,
        );

        self.update_ui(ctx);
        let width = ctx.window_state.logical_size.width;
        if nuon::button()
            .id("freeplay_settings")
            .pos(width - 190.0, 0.0)
            .size(90.0, 30.0)
            .label(if self.settings_open {
                "Close ×"
            } else {
                "Settings"
            })
            .build(&mut self.nuon)
        {
            self.settings_open = !self.settings_open;
        }
        if self.settings_open {
            let previous = ctx.config.visuals();
            super::style::drawer(ctx, &mut self.nuon, 35.0);
            if previous.theme != ctx.config.visuals().theme
                || previous.soft_note_colors != ctx.config.visuals().soft_note_colors
            {
                self.keyboard
                    .set_pressed_by_user_colors(ctx.config.color_schema()[0].clone());
                if let Some(preview) = &mut self.preview {
                    preview.resize(&self.keyboard, ctx);
                }
            }
            self.practice_audio
                .sync_enabled(ctx.config.visuals().practice_sound);
        }

        super::render_nuon(&mut self.nuon, &mut self.nuon_renderer, ctx);
    }

    fn render<'pass>(&'pass mut self, rpass: &mut wgpu_jumpstart::RenderPass<'pass>) {
        self.background.render(rpass);
        self.quad_renderer_bg.render(rpass);
        if let Some(preview) = self.preview.as_mut() {
            preview.render(rpass);
        }
        self.quad_renderer_fg.render(rpass);
        if let Some(glow) = &self.glow {
            glow.render(rpass);
        }
        self.text_renderer.render(rpass);
        self.nuon_renderer.render(rpass);
    }

    fn window_event(&mut self, ctx: &mut Context, event: &WindowEvent) {
        if event.window_resized() || event.scale_factor_changed() {
            self.resize(ctx)
        }

        if event.back_mouse_pressed() || event.key_released(Key::Named(NamedKey::Escape)) {
            if self.settings_open {
                self.settings_open = false;
                return;
            }
            ctx.proxy
                .send_event(TangentEvent::MainMenu(self.song.clone()))
                .ok();
        }

        if event.key_released(Key::Named(NamedKey::Space)) && self.preview.is_some() {
            recorder::toggle_preview_playback(self);
        }

        super::handle_nuon_window_event(&mut self.nuon, event, ctx);
        super::handle_pc_keyboard_to_midi_event(ctx, event);
        super::handle_mouse_to_midi_event(
            &mut self.keyboard,
            &mut self.mouse_to_midi_state,
            ctx,
            event,
            self.settings_open && super::style::drawer_covers_pointer(ctx, 35.0),
        );
    }

    fn midi_event(&mut self, ctx: &mut Context, channel: u8, message: &MidiMessage) {
        self.recorder.push_event(channel, *message);
        self.keyboard.user_midi_event(message);
        ctx.output_manager
            .connection()
            .midi_event(channel.into(), *message);

        match *message {
            MidiMessage::NoteOn { key, vel } => {
                self.live_notes.note_on(channel, key.as_int(), vel.as_int())
            }
            MidiMessage::NoteOff { key, .. } => self.live_notes.note_off(channel, key.as_int()),
            MidiMessage::Controller { controller, value } => {
                self.live_notes
                    .controller(channel, controller.as_int(), value.as_int())
            }
            _ => return,
        }
        self.update_chord_text();
    }
}
