use std::collections::VecDeque;

use crate::{
    TangentEvent, context::Context, output_manager::OutputDescriptor,
    scene::menu_scene::settings::RangeDetection, song::Song,
};

type InputDescriptor = midi_io::MidiInputPort;

pub struct UiState {
    pub(super) library: super::library::LibraryState,
    pub outputs: Vec<OutputDescriptor>,
    pub selected_output: Option<OutputDescriptor>,

    pub inputs: Vec<InputDescriptor>,
    pub selected_input: Option<InputDescriptor>,

    pub is_loading: bool,
    pub arcade_mode: bool,
    pub arcade_error: Option<String>,

    pub song: Option<Song>,

    pub range_detection: RangeDetection,

    page_stack: VecDeque<Page>,
}

impl UiState {
    pub fn new(ctx: &Context, song: Option<Song>) -> Self {
        let mut page_stack = VecDeque::new();
        page_stack.push_front(Page::Main);

        Self {
            library: super::library::LibraryState::new(
                ctx.config.midi_folder().map(|p| p.to_path_buf()),
            ),
            outputs: Vec::new(),
            selected_output: None,
            inputs: Vec::new(),
            selected_input: None,
            is_loading: false,
            arcade_mode: false,
            arcade_error: None,
            song,

            page_stack,
            range_detection: RangeDetection::default(),
        }
    }

    pub fn song(&self) -> Option<&Song> {
        self.song.as_ref()
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    pub fn current(&self) -> &Page {
        self.page_stack.front().unwrap()
    }

    pub fn go_to(&mut self, page: Page) {
        self.page_stack.push_front(page);
    }

    pub fn go_back(&mut self) {
        self.range_detection.stop_detection();
        match self.page_stack.len() {
            1 => {
                // Last page in the stack, let's go to exit page
                self.page_stack.push_front(Page::Exit);
            }
            _ => {
                self.page_stack.pop_front();
            }
        }
    }
}

impl UiState {
    pub fn tick(&mut self, ctx: &mut Context) {
        self.outputs = ctx.output_manager.outputs();
        self.inputs = ctx.input_manager.inputs();

        if self.selected_output.is_none() {
            if let Some(name) = ctx.config.output() {
                if let Some(out) = self
                    .outputs
                    .iter()
                    .find(|output| output.to_string().as_str() == name)
                {
                    self.selected_output = Some(out.clone());
                } else {
                    self.selected_output = self.outputs.first().cloned();
                }
            } else {
                self.selected_output = Some(OutputDescriptor::DummyOutput);
            }
        }

        if self.selected_input.is_none() {
            if let Some(input) = self
                .inputs
                .iter()
                .find(|input| Some(input.to_string().as_str()) == ctx.config.input())
            {
                self.selected_input = Some(input.clone());
            } else {
                self.selected_input = self.inputs.first().cloned();
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Page {
    Exit,
    Main,
    Settings,
    TrackSelection,
}

pub fn connect_io(data: &UiState, ctx: &mut Context) {
    if let Some(out) = data.selected_output.clone() {
        let out = match out {
            #[cfg(feature = "synth")]
            OutputDescriptor::Synth(_) => OutputDescriptor::Synth(ctx.config.soundfont_path()),
            o => o,
        };

        ctx.output_manager.connect(out);
        ctx.output_manager
            .connection()
            .set_gain(ctx.config.audio_gain());
    }

    if let Some(port) = data.selected_input.clone() {
        ctx.input_manager.connect_input(port);
    }
}

pub fn play(data: &mut UiState, ctx: &mut Context) {
    let Some(song) = data.song.as_ref() else {
        return;
    };

    if data.arcade_mode {
        let song = song.clone();
        connect_io(data, ctx);
        ctx.config.save();
        ctx.proxy.send_event(TangentEvent::Arcade(song)).ok();
    } else {
        let song = song.clone();
        connect_io(data, ctx);
        ctx.proxy.send_event(TangentEvent::Play(song)).ok();
    }
}

pub fn freeplay(data: &UiState, ctx: &mut Context) {
    connect_io(data, ctx);

    ctx.proxy
        .send_event(TangentEvent::FreePlay(data.song.clone()))
        .ok();
}
