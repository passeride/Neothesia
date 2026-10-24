use midi_file::MidiTrack;

use crate::context::Context;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlayerConfig {
    Mute,
    Auto,
    Human,
}

#[derive(Debug, Clone)]
pub struct TrackConfig {
    pub track_id: usize,
    pub player: PlayerConfig,
    pub visible: bool,
    pub pitch_range: Option<(u8, u8)>,
}

#[derive(Default, Debug, Clone)]
pub struct SongConfig {
    pub tracks: Box<[TrackConfig]>,
}

impl SongConfig {
    fn new(tracks: &[MidiTrack]) -> Self {
        let tracks: Vec<_> = tracks
            .iter()
            .map(|t| {
                let is_drums = t.has_drums && !t.has_other_than_drums;
                TrackConfig {
                    track_id: t.track_id,
                    player: PlayerConfig::Auto,
                    visible: !is_drums,
                    pitch_range: t.notes.iter().fold(None, |range, note| {
                        Some(match range {
                            None => (note.note, note.note),
                            Some((low, high)) => (low.min(note.note), high.max(note.note)),
                        })
                    }),
                }
            })
            .collect();
        Self {
            tracks: tracks.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Song {
    pub source_path: Option<std::path::PathBuf>,
    pub source_bytes: u64,
    pub source_modified: Option<u64>,
    pub file: midi_file::MidiFile,
    pub config: SongConfig,
}

impl Song {
    pub fn new(file: midi_file::MidiFile) -> Self {
        let config = SongConfig::new(&file.tracks);
        Self {
            file,
            config,
            source_path: None,
            source_bytes: 0,
            source_modified: None,
        }
    }

    pub fn set_source(&mut self, path: std::path::PathBuf) {
        let path = std::path::absolute(&path).unwrap_or(path);
        let meta = std::fs::metadata(&path).ok();
        self.source_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        self.source_modified = crate::records::modified(meta.and_then(|m| m.modified().ok()));
        self.source_path = Some(path);
    }

    pub fn from_env(ctx: &Context) -> Option<Self> {
        let args: Vec<String> = std::env::args().collect();
        let midi_file = if args.len() > 1 {
            midi_file::MidiFile::new(&args[1]).ok()
        } else if let Some(last) = ctx.config.last_opened_song() {
            midi_file::MidiFile::new(last).ok()
        } else {
            None
        };

        let mut song = Self::new(midi_file?);
        let source = if args.len() > 1 {
            Some(args[1].clone().into())
        } else {
            ctx.config.last_opened_song().cloned()
        };
        if let Some(path) = source {
            song.set_source(path);
        }
        Some(song)
    }
}
