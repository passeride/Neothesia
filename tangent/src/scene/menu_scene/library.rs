use std::path::{Path, PathBuf};

use tangent_core::midi_library::{Catalog, SortOrder};
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{Key, NamedKey},
};

use super::{
    MenuScene, MsgFn, UiState, on_async,
    state::{self, Page},
};
use crate::{
    context::Context,
    locals,
    song::Song,
    utils::{BoxFuture, task::thread},
};

#[derive(Default)]
pub(super) struct LibraryState {
    pub folder: Option<PathBuf>,
    catalog: Catalog,
    busy: bool,
    pub needs_scan: bool,
    query: String,
    search_focus: bool,
    select_all: bool,
    order: SortOrder,
    page: usize,
    error: Option<String>,
    // Rebuild the filtered index only when the query, order or scan changes.
    matches: Vec<usize>,
}
impl LibraryState {
    pub fn new(folder: Option<PathBuf>) -> Self {
        Self {
            folder,
            ..Default::default()
        }
    }
    fn filter(&mut self) {
        self.matches = self.catalog.filtered(&self.query, self.order);
        self.page = 0;
    }
    fn finish(&mut self, result: Result<Catalog, String>) {
        self.busy = false;
        match result {
            Ok(catalog) => {
                self.catalog = catalog;
                self.error = None;
            }
            Err(error) => {
                self.catalog = Catalog::default();
                self.error = Some(error);
            }
        }
        self.filter();
    }
}
async fn scan(folder: PathBuf, recursive: bool) -> Result<Catalog, String> {
    thread::spawn("midi-library-scan".to_owned(), move || {
        Catalog::scan(&folder, recursive).map_err(|e| format!("Cannot read this folder: {e}"))
    })
    .join()
    .await
    .map_err(|_| "Folder scan did not finish. Try Refresh.".to_owned())?
}

impl MenuScene {
    pub(super) fn refresh_library(&mut self, ctx: &Context) {
        if self.state.library.busy {
            return;
        }
        let Some(folder) = self.state.library.folder.clone() else {
            return;
        };
        let recursive = ctx.config.include_subfolders();
        self.state.library.busy = true;
        self.futures
            .push(on_async(scan(folder, recursive), |result, state, _| {
                state.library.finish(result);
            }));
    }
    fn choose_library_folder(&mut self, _ctx: &Context) {
        if self.state.library.busy {
            return;
        }
        self.state.library.busy = true;
        let previous = self.state.library.folder.clone();
        self.futures.push(on_async(
            async move {
                let mut dialog =
                    rfd::AsyncFileDialog::new().set_title("Choose your MIDI library folder");
                if let Some(folder) = previous {
                    dialog = dialog.set_directory(folder);
                }
                dialog
                    .pick_folder()
                    .await
                    .map(|folder| folder.path().to_path_buf())
            },
            |folder, state, ctx| {
                state.library.busy = false;
                if let Some(folder) = folder {
                    ctx.config.set_midi_folder(Some(folder.clone()));
                    ctx.config.save();
                    state.library.folder = Some(folder);
                    state.library.query.clear();
                    state.library.catalog = Catalog::default();
                    state.library.error = None;
                    state.library.filter();
                    state.library.needs_scan = true;
                }
            },
        ));
    }

    fn open_library_song(&mut self, path: PathBuf) {
        if self.state.is_loading {
            return;
        }
        self.state.is_loading = true;
        self.state.library.error = None;
        self.futures.push(load_song(path, false));
    }
    pub(super) fn library_keyboard(&mut self, ctx: &Context, event: &WindowEvent) -> bool {
        if event_left_pressed(event) {
            self.state.library.search_focus = false;
        }
        if !self.state.library.search_focus {
            let command = ctx.window_state.modifiers_state.control_key()
                || ctx.window_state.modifiers_state.super_key();
            if let WindowEvent::KeyboardInput { event, .. } = event
                && event.state == ElementState::Pressed
                && command
                && matches!(&event.logical_key, Key::Character(c) if c.eq_ignore_ascii_case("l"))
            {
                self.state.library.search_focus = true;
                self.state.library.select_all = true;
                return true;
            }
            return false;
        }
        let WindowEvent::KeyboardInput { event, .. } = event else {
            return false;
        };
        if event.state != ElementState::Pressed {
            return true;
        }
        let l = &mut self.state.library;
        let command = ctx.window_state.modifiers_state.control_key()
            || ctx.window_state.modifiers_state.super_key();
        match &event.logical_key {
            Key::Named(NamedKey::Escape) => {
                l.search_focus = false;
                l.select_all = false;
            }
            Key::Named(NamedKey::Tab) => {
                l.search_focus = false;
                l.select_all = false;
            }
            Key::Named(NamedKey::Enter) => {
                let path = l
                    .matches
                    .first()
                    .map(|i| l.catalog.entries[*i].path.clone());
                if let Some(path) = path {
                    self.open_library_song(path);
                }
            }
            Key::Named(NamedKey::Backspace) => {
                if command || l.select_all {
                    l.query.clear();
                } else {
                    l.query.pop();
                }
                l.select_all = false;
                l.filter();
            }
            Key::Character(c)
                if command && (c.eq_ignore_ascii_case("a") || c.eq_ignore_ascii_case("l")) =>
            {
                l.select_all = true
            }
            _ if !command && !ctx.window_state.modifiers_state.alt_key() => {
                let input = event.text.as_deref().filter(|s| !s.is_empty()).or_else(|| {
                    if let Key::Character(c) = &event.logical_key {
                        Some(c.as_str())
                    } else {
                        None
                    }
                });
                if let Some(text) = input
                    && !text.chars().any(char::is_control)
                {
                    if l.select_all {
                        l.query.clear();
                        l.select_all = false;
                    }
                    let remaining = 256usize.saturating_sub(l.query.chars().count());
                    l.query.extend(text.chars().take(remaining));
                    l.filter();
                }
            }
            _ => {}
        }
        true
    }

    pub(super) fn library_ui(&mut self, ctx: &mut Context, ui: &mut nuon::Ui) {
        let width = ctx.window_state.logical_size.width;
        let height = ctx.window_state.logical_size.height;
        let margin = 24.0;
        let body = (width - margin * 2.0).max(0.0);
        // Compact branding leaves the useful part of the home screen to songs.
        nuon::image(self.logo)
            .pos(margin, 16.0)
            .size(220.0, 40.0)
            .build(ui);
        if action(
            ui,
            "library_settings",
            (width - 116.0, 22.0),
            (92.0, 32.0),
            &locals::menu::settings(),
            true,
        ) {
            self.state.library.search_focus = false;
            self.state.go_to(Page::Settings);
        }
        if action(
            ui,
            "library_exit",
            (width - 194.0, 22.0),
            (68.0, 32.0),
            &locals::menu::exit(),
            true,
        ) {
            self.state.go_back();
        }
        text(
            ui,
            margin,
            78.0,
            body,
            35.0,
            28.0,
            "Your music library",
            false,
        );
        let busy = self.state.library.busy;
        let folder = self
            .state
            .library
            .folder
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned());
        text(
            ui,
            margin,
            117.0,
            body,
            22.0,
            13.0,
            &shorten(
                folder
                    .as_deref()
                    .unwrap_or("A home for the pieces you want to play."),
                (body / 7.0) as usize,
            ),
            true,
        );
        let row_y = 151.0;
        if action(
            ui,
            "library_folder",
            (margin, row_y),
            (138.0, 34.0),
            if folder.is_some() {
                "Change folder"
            } else {
                "Choose folder"
            },
            !busy,
        ) {
            self.choose_library_folder(ctx);
        }
        if action(
            ui,
            "library_refresh",
            (margin + 148.0, row_y),
            (85.0, 34.0),
            "Refresh",
            !busy && folder.is_some(),
        ) {
            self.refresh_library(ctx);
        }
        if action(
            ui,
            "library_file",
            (margin + 243.0, row_y),
            (112.0, 34.0),
            &locals::menu::select_file(),
            !busy,
        ) {
            self.futures
                .push(super::midi_picker::open_midi_file_picker(&mut self.state));
        }
        let (option_y, filter_y) = if body < 610.0 {
            (195.0, 238.0)
        } else {
            (row_y, 200.0)
        };
        let option_x = if body < 610.0 { margin } else { margin + 370.0 };
        let include = ctx.config.include_subfolders();
        if action(
            ui,
            "library_recursive",
            (option_x, option_y),
            (185.0, 34.0),
            if include {
                "Subfolders: Included"
            } else {
                "Subfolders: Excluded"
            },
            !busy,
        ) {
            ctx.config.set_include_subfolders(!include);
            ctx.config.save();
            self.refresh_library(ctx);
        }
        let search_w = (body - 215.0).max(100.0);
        let field = nuon::click_area("library_search")
            .pos(margin, filter_y)
            .size(search_w, 38.0)
            .build(ui);
        if field.is_clicked() {
            self.state.library.search_focus = true;
            self.state.library.select_all = false;
        }
        nuon::quad()
            .pos(margin, filter_y)
            .size(search_w, 38.0)
            .border_radius([9.0; 4])
            .color(ui.theme.surface)
            .border_width(1.0)
            .border_color(if self.state.library.search_focus {
                ui.theme.accent
            } else {
                ui.theme.border
            })
            .build(ui);
        let search = if self.state.library.query.is_empty() {
            "Search pieces or folders…  (Ctrl+L)".to_owned()
        } else {
            format!(
                "{}{}",
                self.state.library.query,
                if self.state.library.search_focus {
                    "│"
                } else {
                    ""
                }
            )
        };
        text(
            ui,
            margin + 12.0,
            filter_y,
            search_w - 24.0,
            38.0,
            14.0,
            &shorten(&search, ((search_w - 24.0) / 8.0) as usize),
            self.state.library.query.is_empty(),
        );
        if action(
            ui,
            "library_clear_search",
            (margin + search_w + 8.0, filter_y),
            (65.0, 38.0),
            "Clear",
            !self.state.library.query.is_empty(),
        ) {
            self.state.library.query.clear();
            self.state.library.filter();
        }
        if action(
            ui,
            "library_sort",
            (margin + search_w + 81.0, filter_y),
            (134.0, 38.0),
            if self.state.library.order == SortOrder::Name {
                "Sort: Name"
            } else {
                "Sort: Newest"
            },
            true,
        ) {
            self.state.library.order = if self.state.library.order == SortOrder::Name {
                SortOrder::Newest
            } else {
                SortOrder::Name
            };
            self.state.library.filter();
        }
        let content_y = filter_y + 82.0;
        let bottom = height - 100.0;
        let available = (bottom - content_y - 46.0).max(0.0);
        let columns = ((body + 12.0) / 285.0).floor().max(1.0) as usize;
        let cover_h = if available >= 204.0 {
            100.0
        } else if available >= 144.0 {
            40.0
        } else {
            0.0
        };
        let card_h = 104.0 + cover_h;
        let stride = card_h + 12.0;
        let rows = ((available + 12.0) / stride).floor().max(1.0) as usize;
        let per_page = columns * rows;
        let l = &mut self.state.library;
        let total = l.matches.len();
        let pages = total.div_ceil(per_page).max(1);
        l.page = l.page.min(pages - 1);
        let status = if busy {
            "Scanning folder…".to_owned()
        } else if let Some(error) = &l.error {
            error.clone()
        } else if let Some(warning) = &ctx.records.warning {
            warning.clone()
        } else if l.catalog.skipped > 0 {
            format!(
                "{} pieces · {} entries could not be read",
                total, l.catalog.skipped
            )
        } else {
            format!(
                "{} piece{}{}",
                total,
                if total == 1 { "" } else { "s" },
                if !l.query.is_empty() { " found" } else { "" }
            )
        };
        text(
            ui,
            margin,
            filter_y + 46.0,
            body,
            26.0,
            13.0,
            &shorten(&status, (body / 7.0) as usize),
            true,
        );
        let mut open = None;
        let mut open_arcade = None;
        if total == 0 && !busy && l.error.is_none() {
            let (title, hint) = if l.folder.is_none() {
                (
                    "Keep your music within reach",
                    "Choose a folder once. Its MIDI and MusicXML files will appear here every time you open Tangent.",
                )
            } else if l.catalog.entries.is_empty() {
                (
                    "Your next piece belongs here",
                    "Add .mid or .midi files to this folder, then choose Refresh.",
                )
            } else {
                (
                    "No matching pieces",
                    "Try a different title or folder name, or clear the search.",
                )
            };
            let y = content_y + (available * 0.25).max(8.0);
            text(ui, margin, y, body, 40.0, 22.0, title, false);
            text(ui, margin, y + 48.0, body, 55.0, 14.0, hint, true);
        }
        let card_w = ((body - (columns - 1) as f32 * 12.0) / columns as f32).max(1.0);
        // Only this page is shaped/rendered; large catalogs do not create one
        // text buffer per file on every frame.
        if available >= card_h {
            for (cell, index) in l
                .matches
                .iter()
                .skip(l.page * per_page)
                .take(per_page)
                .enumerate()
            {
                let entry = &l.catalog.entries[*index];
                let x = margin + (cell % columns) as f32 * (card_w + 12.0);
                let y = content_y + (cell / columns) as f32 * stride;
                let event = nuon::click_area(("library_song", &entry.path))
                    .pos(x, y)
                    .size(card_w, card_h - 32.0)
                    .build(ui);
                let selected = ctx
                    .config
                    .last_opened_song()
                    .is_some_and(|p| p == &entry.path);
                nuon::quad()
                    .pos(x, y)
                    .size(card_w, card_h)
                    .border_radius([12.0; 4])
                    .color(if event.is_hovered() {
                        ui.theme.hover
                    } else {
                        ui.theme.surface
                    })
                    .border_color(if selected {
                        ui.theme.accent
                    } else {
                        ui.theme.border
                    })
                    .border_width(1.0)
                    .build(ui);
                let chars = ((card_w - 32.0) / 8.0) as usize;
                if cover_h > 0.0 {
                    // A deterministic procedural cover, drawn only for this page.
                    for (n, byte) in entry.title.bytes().take(12).enumerate() {
                        let bar_h = (22.0 + (byte % 50) as f32).min(cover_h - 12.0);
                        let mut accent = ui.theme.accent;
                        accent.a = 0.12;
                        nuon::quad()
                            .pos(
                                x + 16.0 + n as f32 * ((card_w - 40.0) / 12.0),
                                y + cover_h - 10.0 - bar_h,
                            )
                            .size(8.0, bar_h)
                            .border_radius([4.0; 4])
                            .color(accent)
                            .build(ui);
                    }
                    if let Some(record) = ctx.records.for_entry(entry) {
                        text(
                            ui,
                            x + 16.0,
                            y + 8.0,
                            card_w - 32.0,
                            32.0,
                            if cover_h > 40.0 { 22.0 } else { 16.0 },
                            &format!(
                                "BEST {:07}{}",
                                record.score,
                                if record.full_combo { " · FC" } else { "" }
                            ),
                            false,
                        );
                        if cover_h > 40.0 {
                            text(
                                ui,
                                x + 16.0,
                                y + 50.0,
                                card_w - 32.0,
                                30.0,
                                12.0,
                                &shorten(&record.category(), chars),
                                true,
                            );
                        }
                    } else {
                        text(
                            ui,
                            x + 16.0,
                            y + 8.0,
                            card_w - 32.0,
                            26.0,
                            14.0,
                            "ARCADE · UNPLAYED",
                            true,
                        );
                    }
                    nuon::quad()
                        .pos(x + 16.0, y + cover_h - 2.0)
                        .size(card_w - 32.0, 1.0)
                        .color(ui.theme.border)
                        .build(ui);
                }
                text(
                    ui,
                    x + 16.0,
                    y + 12.0 + cover_h,
                    card_w - 32.0,
                    38.0,
                    16.0,
                    &shorten(&entry.title, chars * 2),
                    false,
                );
                let parent = entry
                    .relative_path
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or(Path::new("Library"));
                text(
                    ui,
                    x + 16.0,
                    y + 52.0 + cover_h,
                    card_w - 32.0,
                    20.0,
                    12.0,
                    &shorten(
                        &if cover_h <= 40.0 {
                            ctx.records
                                .for_entry(entry)
                                .map(|r| format!("{:07} · {}", r.score, r.category()))
                                .unwrap_or_else(|| parent.to_string_lossy().into_owned())
                        } else {
                            parent.to_string_lossy().into_owned()
                        },
                        chars,
                    ),
                    true,
                );
                text(
                    ui,
                    x + 16.0,
                    y + 77.0 + cover_h,
                    card_w - 165.0,
                    18.0,
                    12.0,
                    &file_size(entry.bytes),
                    true,
                );
                if nuon::button()
                    .id(("card-arcade", &entry.path))
                    .pos(x + card_w - 146.0, y + card_h - 28.0)
                    .size(76.0, 23.0)
                    .label("Arcade")
                    .border_radius([6.0; 4])
                    .build(ui)
                {
                    open_arcade = Some(entry.path.clone());
                }
                if nuon::button()
                    .id(("card-play", &entry.path))
                    .pos(x + card_w - 62.0, y + card_h - 28.0)
                    .size(48.0, 23.0)
                    .label("Play")
                    .border_radius([6.0; 4])
                    .build(ui)
                {
                    open = Some(entry.path.clone());
                }
                if event.is_clicked() {
                    open = Some(entry.path.clone());
                }
            }
        } else if total > 0 {
            text(
                ui,
                margin,
                content_y,
                body,
                30.0,
                14.0,
                "Enlarge the window to see your pieces.",
                true,
            );
        }
        if total > per_page {
            let y = bottom - 36.0;
            if action(
                ui,
                "library_previous",
                (margin, y),
                (92.0, 32.0),
                "Previous",
                l.page > 0,
            ) {
                l.page -= 1;
            }
            text(
                ui,
                margin + 104.0,
                y,
                145.0,
                32.0,
                13.0,
                &format!("Page {} / {}", l.page + 1, pages),
                true,
            );
            if action(
                ui,
                "library_next",
                (margin + 261.0, y),
                (92.0, 32.0),
                "Next",
                l.page + 1 < pages,
            ) {
                l.page += 1;
            }
        }
        // Keep the existing transport entry points for the last selected song.
        nuon::quad()
            .pos(0.0, height - 78.0)
            .size(width, 78.0)
            .color(ui.theme.surface)
            .build(ui);
        if action(
            ui,
            "library_freeplay",
            (margin, height - 60.0),
            (110.0, 38.0),
            &locals::menu::freeplay(),
            true,
        ) {
            state::freeplay(&self.state, ctx);
        }
        let song_name = self
            .state
            .song()
            .map(|s| s.file.name.as_str())
            .unwrap_or("Choose a piece to begin");
        text(
            ui,
            margin + 125.0,
            height - 60.0,
            (body - 435.0).max(0.0),
            38.0,
            13.0,
            &shorten(song_name, ((body - 435.0).max(0.0) / 7.0) as usize),
            true,
        );
        if action(
            ui,
            "library_arcade",
            (width - margin - 288.0, height - 60.0),
            (90.0, 38.0),
            "Arcade",
            self.state.song.is_some(),
        ) {
            self.state.arcade_mode = true;
            self.state.arcade_error = None;
            self.state.go_to(Page::TrackSelection);
        }
        if action(
            ui,
            "library_tracks",
            (width - margin - 188.0, height - 60.0),
            (88.0, 38.0),
            &locals::menu::tracks(),
            self.state.song.is_some(),
        ) {
            self.state.arcade_mode = false;
            self.state.go_to(Page::TrackSelection);
        }
        if action(
            ui,
            "library_play",
            (width - margin - 90.0, height - 60.0),
            (90.0, 38.0),
            &locals::menu::play(),
            self.state.song.is_some(),
        ) {
            self.state.arcade_mode = false;
            state::play(&mut self.state, ctx);
        }
        if let Some(path) = open {
            self.open_library_song(path);
        } else if let Some(path) = open_arcade
            && !self.state.is_loading
        {
            self.state.is_loading = true;
            self.state.library.error = None;
            self.futures.push(load_song(path, true));
        }
    }
}

pub(super) async fn read_song(path: PathBuf) -> Result<midi_file::MidiFile, String> {
    thread::spawn("midi-library-open".to_owned(), move || {
        midi_file::MidiFile::new(&path).map_err(|e| e.to_string())
    })
    .join()
    .await
    .unwrap_or_else(|_| Err("Score loading did not finish".to_owned()))
}
pub(super) fn finish_song(
    state: &mut UiState,
    ctx: &mut Context,
    path: PathBuf,
    result: Result<midi_file::MidiFile, String>,
    play: bool,
) {
    state.is_loading = false;
    match result {
        Ok(midi) => {
            ctx.config.set_last_opened_song(Some(path.clone()));
            ctx.config.save();
            let imported_score = midi.score.is_some();
            let mut song = Song::new(midi);
            song.set_source(path);
            state.song = Some(song);
            state.library.error = None;
            if play {
                state.arcade_mode = false;
                if imported_score {
                    state.go_to(Page::TrackSelection);
                } else {
                    state::play(state, ctx);
                }
            }
        }
        Err(error) => {
            state.library.error = Some(format!(
                "Could not open {}: {error}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ))
        }
    }
}
fn load_song(path: PathBuf, arcade: bool) -> BoxFuture<MsgFn> {
    on_async(
        async move {
            let result = read_song(path.clone()).await;
            (path, result)
        },
        move |(path, result), state, ctx| {
            let valid = result.is_ok();
            finish_song(state, ctx, path, result, !arcade);
            if arcade && valid {
                state.arcade_mode = true;
                state.arcade_error = None;
                state.go_to(Page::TrackSelection);
            }
        },
    )
}

fn event_left_pressed(event: &WindowEvent) -> bool {
    matches!(
        event,
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: winit::event::MouseButton::Left,
            ..
        }
    )
}
fn action(
    ui: &mut nuon::Ui,
    id: &'static str,
    pos: (f32, f32),
    size: (f32, f32),
    title: &str,
    enabled: bool,
) -> bool {
    nuon::button()
        .id(id)
        .pos(pos.0, pos.1)
        .size(size.0, size.1)
        .label(title.to_owned())
        .font_color(if enabled {
            nuon::Color::WHITE
        } else {
            ui.theme.muted
        })
        .build(ui)
        && enabled
}
#[allow(clippy::too_many_arguments)]
fn text(ui: &mut nuon::Ui, x: f32, y: f32, w: f32, h: f32, size: f32, value: &str, muted: bool) {
    nuon::label()
        .pos(x, y)
        .size(w, h)
        .font_size(size)
        .text(value)
        .text_justify(nuon::TextAlign::Start)
        .color(if muted {
            ui.theme.muted
        } else {
            nuon::Color::WHITE
        })
        .build(ui);
}
fn shorten(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_owned()
    } else if max < 2 {
        String::new()
    } else {
        text.chars().take(max - 1).chain(['…']).collect()
    }
}
fn file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
