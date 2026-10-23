//! A folder-backed MIDI catalog. Scanning reads filesystem metadata only; MIDI
//! contents are parsed on demand by the application when a song is opened.
use std::{
    cmp::Reverse,
    fs, io,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Clone, Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub title: String,
    pub bytes: u64,
    pub modified: Option<SystemTime>,
    search_text: String,
}
impl Entry {
    pub fn matches(&self, query: &str) -> bool {
        query
            .split_whitespace()
            .all(|word| self.search_text.contains(&word.to_lowercase()))
    }
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum SortOrder {
    #[default]
    Name,
    Newest,
}
#[derive(Default)]
pub struct Catalog {
    pub entries: Vec<Entry>,
    pub skipped: usize,
}
impl Catalog {
    /// Iterative traversal avoids stack overflows in deep trees. Directory and
    /// file symlinks are skipped, so a library cannot loop or escape via links.
    /// An unavailable root is an error; unreadable descendants are reported.
    pub fn scan(root: &Path, recursive: bool) -> io::Result<Self> {
        let first = fs::read_dir(root)?;
        let mut pending = vec![first];
        let mut result = Self::default();
        while let Some(directory) = pending.pop() {
            for item in directory {
                let item = match item {
                    Ok(item) => item,
                    Err(_) => {
                        result.skipped += 1;
                        continue;
                    }
                };
                let kind = match item.file_type() {
                    Ok(kind) => kind,
                    Err(_) => {
                        result.skipped += 1;
                        continue;
                    }
                };
                let path = item.path();
                if kind.is_dir() && recursive {
                    match fs::read_dir(&path) {
                        Ok(dir) => pending.push(dir),
                        Err(_) => result.skipped += 1,
                    }
                } else if kind.is_file() && is_score_file(&path) {
                    let relative_path = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
                    let metadata = item.metadata().ok();
                    result.entries.push(Entry {
                        title: path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                        search_text: relative_path.to_string_lossy().to_lowercase(),
                        relative_path,
                        path,
                        bytes: metadata.as_ref().map(|m| m.len()).unwrap_or(0),
                        modified: metadata.and_then(|m| m.modified().ok()),
                    });
                }
            }
        }
        result.entries.sort_by(|a, b| {
            a.title
                .to_lowercase()
                .cmp(&b.title.to_lowercase())
                .then(a.relative_path.cmp(&b.relative_path))
        });
        Ok(result)
    }
    /// Return indices into the catalog rather than duplicating paths/metadata.
    pub fn filtered(&self, query: &str, order: SortOrder) -> Vec<usize> {
        let mut indices: Vec<_> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.matches(query))
            .map(|(i, _)| i)
            .collect();
        if order == SortOrder::Newest {
            indices.sort_by_key(|i| Reverse(self.entries[*i].modified));
        }
        indices
    }
}
fn is_score_file(path: &Path) -> bool {
    path.extension().and_then(|s| s.to_str()).is_some_and(|s| {
        ["mid", "midi", "musicxml", "xml", "mxl"]
            .iter()
            .any(|ext| s.eq_ignore_ascii_case(ext))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "tangent-library-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
        fn file(&self, name: &str) {
            let p = self.0.join(name);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, b"metadata only").unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).ok();
        }
    }
    #[test]
    fn scans_mixed_extensions_and_subfolders_without_parsing() {
        let dir = Fixture::new();
        for p in [
            "zebra.MID",
            "Alpha.midi",
            "Bach/Prelude.mid",
            "Bach/readme.txt",
            "wrong.mid.txt",
        ] {
            dir.file(p);
        }
        assert_eq!(Catalog::scan(&dir.0, false).unwrap().entries.len(), 2);
        let all = Catalog::scan(&dir.0, true).unwrap();
        assert_eq!(
            all.entries
                .iter()
                .map(|e| e.title.as_str())
                .collect::<Vec<_>>(),
            ["Alpha", "Prelude", "zebra"]
        );
        assert_eq!(all.entries[1].relative_path, Path::new("Bach/Prelude.mid"));
        assert!(all.entries.iter().all(|e| e.bytes > 0));
    }
    #[test]
    fn scans_musicxml_extensions_without_treating_other_formats_as_scores() {
        let dir = Fixture::new();
        for p in ["a.musicxml", "b.MXL", "c.xml", "d.json", "e.pdf"] {
            dir.file(p);
        }
        assert_eq!(Catalog::scan(&dir.0, false).unwrap().entries.len(), 3);
    }
    #[test]
    fn rescan_observes_added_removed_and_renamed_files() {
        let dir = Fixture::new();
        dir.file("old.mid");
        assert_eq!(Catalog::scan(&dir.0, true).unwrap().entries.len(), 1);
        fs::rename(dir.0.join("old.mid"), dir.0.join("renamed.MIDI")).unwrap();
        dir.file("new.mid");
        assert_eq!(Catalog::scan(&dir.0, true).unwrap().entries.len(), 2);
        fs::remove_file(dir.0.join("new.mid")).unwrap();
        assert_eq!(
            Catalog::scan(&dir.0, true).unwrap().entries[0].title,
            "renamed"
        );
        assert!(Catalog::scan(&dir.0.join("missing"), true).is_err());
        assert!(Catalog::scan(&dir.0.join("renamed.MIDI"), true).is_err());
    }
    #[test]
    fn search_handles_unicode_folders_duplicate_titles_and_stable_sorting() {
        let dir = Fixture::new();
        for p in ["Øving/BÅDE.mid", "Bach/Prelude.mid", "Chopin/Prelude.mid"] {
            dir.file(p);
        }
        let mut all = Catalog::scan(&dir.0, true).unwrap();
        assert_eq!(all.filtered("øving både", SortOrder::Name).len(), 1);
        assert_eq!(all.filtered("prelude", SortOrder::Name).len(), 2);
        assert_eq!(all.filtered("BACH prelude", SortOrder::Name).len(), 1);
        assert!(all.filtered("missing", SortOrder::Name).is_empty());
        all.entries[0].modified = Some(SystemTime::UNIX_EPOCH);
        all.entries[1].modified = Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1));
        all.entries[2].modified = None;
        assert_eq!(all.filtered("", SortOrder::Newest), [1, 0, 2]);
    }
    #[cfg(unix)]
    #[test]
    fn symlink_cycles_and_linked_files_are_not_followed() {
        let dir = Fixture::new();
        dir.file("nested/real.mid");
        std::os::unix::fs::symlink(&dir.0, dir.0.join("nested/loop")).unwrap();
        std::os::unix::fs::symlink(dir.0.join("nested/real.mid"), dir.0.join("linked.mid"))
            .unwrap();
        assert_eq!(Catalog::scan(&dir.0, true).unwrap().entries.len(), 1);
    }
}
