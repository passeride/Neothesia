use super::{UiState, library};
use crate::{
    scene::menu_scene::{MsgFn, on_async},
    utils::BoxFuture,
};

pub fn open_midi_file_picker(data: &mut UiState) -> BoxFuture<MsgFn> {
    data.is_loading = true;
    on_async(
        async {
            let file = rfd::AsyncFileDialog::new()
                .add_filter(
                    "Piano scores (MIDI / MusicXML)",
                    &[
                        "mid", "midi", "musicxml", "xml", "mxl", "MID", "MIDI", "MUSICXML", "XML",
                        "MXL",
                    ],
                )
                .pick_file()
                .await?;
            let path = file.path().to_path_buf();
            let result = library::read_song(path.clone()).await;
            Some((path, result))
        },
        |choice, data, ctx| {
            data.is_loading = false;
            if let Some((path, result)) = choice {
                library::finish_song(data, ctx, path, result, false);
            }
        },
    )
}
