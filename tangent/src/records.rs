//! Immutable completed-run files: commit with rename to a unique destination,
//! never overwrite an earlier record. Settings and song files are separate.
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub schema: u32,
    pub rules: u32,
    pub key: String,
    pub score: u32,
    pub accuracy_milli: u32,
    pub full_combo: bool,
    pub extras: u64,
    pub tracks: Vec<usize>,
    pub speed_milli: u32,
    pub source: Option<PathBuf>,
    pub source_bytes: u64,
    pub source_modified: Option<u64>,
    pub created: u64,
}
impl Record {
    pub fn category(&self) -> String {
        format!(
            "Original · {}% · T{}",
            self.speed_milli as f64 / 10.0,
            self.tracks
                .iter()
                .map(|t| (t + 1).to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
pub fn modified(time: Option<SystemTime>) -> Option<u64> {
    time.and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|t| u64::try_from(t.as_nanos()).ok())
}

#[derive(Default)]
pub struct Records {
    folder: Option<PathBuf>,
    best: HashMap<String, Record>,
    locations: HashMap<PathBuf, Vec<(u64, Option<u64>, String)>>,
    /// Associates catalog paths with exact categories; contents are revalidated
    /// when opened. Gallery never parses every MIDI merely to render badges.
    pub warning: Option<String>,
}
impl Records {
    pub fn load() -> Self {
        let folder = tangent_core::utils::resources::settings_ron()
            .and_then(|p| p.parent().map(|p| p.join("arcade-runs")));
        Self::load_folder(folder)
    }
    fn load_folder(folder: Option<PathBuf>) -> Self {
        let mut store = Self {
            folder,
            ..Default::default()
        };
        if let Some(folder) = store.folder.as_ref() {
            match fs::read_dir(folder) {
                Ok(files) => {
                    let paths: Vec<_> = files.filter_map(Result::ok).map(|f| f.path()).collect();
                    for path in paths {
                        if path.extension().is_none_or(|e| e != "ron") {
                            continue;
                        }
                        if fs::metadata(&path).is_ok_and(|m| m.len() > 65_536) {
                            store.warning = Some("An oversized Arcade record was skipped.".into());
                            continue;
                        }
                        let record = fs::read_to_string(&path)
                            .ok()
                            .and_then(|s| ron::from_str::<Record>(&s).ok());
                        if let Some(r) = record.filter(|r| {
                            r.schema == 1
                                && r.rules == tangent_core::arcade::RULES_VERSION
                                && r.score <= 1_000_000
                                && r.accuracy_milli <= 100_000
                                && r.key.len() == 64
                        }) {
                            store.insert(r);
                        } else {
                            store.warning = Some("An unreadable Arcade record was skipped.".into());
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => store.warning = Some(format!("Cannot read Arcade records: {e}")),
            }
        }
        store
    }
    fn insert(&mut self, record: Record) {
        if let Some(path) = &record.source {
            let aliases = self.locations.entry(path.clone()).or_default();
            if !aliases.iter().any(|(bytes, modified, key)| {
                *bytes == record.source_bytes
                    && *modified == record.source_modified
                    && key == &record.key
            }) {
                aliases.push((
                    record.source_bytes,
                    record.source_modified,
                    record.key.clone(),
                ));
            }
        }
        let replace = self.best.get(&record.key).is_none_or(|r| {
            (
                record.score,
                record.accuracy_milli,
                std::cmp::Reverse(record.extras),
                record.full_combo,
                std::cmp::Reverse(record.created),
            ) > (
                r.score,
                r.accuracy_milli,
                std::cmp::Reverse(r.extras),
                r.full_combo,
                std::cmp::Reverse(r.created),
            )
        });
        if replace {
            self.best.insert(record.key.clone(), record);
        }
    }
    pub fn best(&self, key: &str) -> Option<&Record> {
        self.best.get(key)
    }
    pub fn for_entry(&self, entry: &tangent_core::midi_library::Entry) -> Option<&Record> {
        self.locations
            .get(&entry.path)?
            .iter()
            .filter(|(bytes, time, _)| *bytes == entry.bytes && *time == modified(entry.modified))
            .filter_map(|(_, _, key)| self.best.get(key))
            .max_by_key(|r| (r.score, r.accuracy_milli, std::cmp::Reverse(&r.key)))
    }
    pub fn save(&mut self, record: Record) -> io::Result<()> {
        let folder = self
            .folder
            .as_deref()
            .ok_or_else(|| io::Error::other("No writable settings location"))?;
        commit(folder, &record)?;
        self.insert(record);
        Ok(())
    }
}
fn commit(folder: &Path, record: &Record) -> io::Result<()> {
    fs::create_dir_all(folder)?;
    let data = ron::ser::to_string(record).map_err(io::Error::other)?;
    for suffix in 0..100 {
        let stem = format!("{}-{}-{suffix}", record.created, std::process::id());
        let temp = folder.join(format!(".{stem}.tmp"));
        let dest = folder.join(format!("{stem}.ron"));
        if dest.exists() {
            continue;
        }
        let mut f = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
        {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        };
        let result = (|| {
            f.write_all(data.as_bytes())?;
            f.sync_all()?;
            drop(f);
            fs::rename(&temp, &dest)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        return result;
    }
    Err(io::Error::other("Could not allocate a unique run filename"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn record(score: u32) -> Record {
        Record {
            schema: 1,
            rules: 1,
            key: "a".repeat(64),
            score,
            accuracy_milli: 99_000,
            full_combo: false,
            extras: 0,
            tracks: vec![1],
            speed_milli: 1000,
            source: None,
            source_bytes: 0,
            source_modified: None,
            created: 1,
        }
    }
    #[test]
    fn immutable_commits_preserve_previous_runs_and_ignore_partial_files() {
        let dir = std::env::temp_dir().join(format!(
            "tangent-records-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        commit(&dir, &record(100)).unwrap();
        commit(&dir, &record(200)).unwrap();
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);
        fs::write(dir.join(".unfinished.tmp"), "partial").unwrap();
        let loaded = Records::load_folder(Some(dir.clone()));
        assert_eq!(loaded.best(&"a".repeat(64)).unwrap().score, 200);
        assert!(loaded.warning.is_none());
        let mut store = Records::default();
        store.insert(record(200));
        store.insert(record(100));
        assert_eq!(store.best(&"a".repeat(64)).unwrap().score, 200);
        assert!(commit(&dir.join("not-a-dir"), &record(300)).is_ok());
        fs::write(dir.join("blocked"), "x").unwrap();
        assert!(commit(&dir.join("blocked"), &record(300)).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn a_lower_score_at_a_new_location_keeps_the_best_after_reload() {
        let dir = std::env::temp_dir().join(format!(
            "tangent-record-alias-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let old = dir.join("old.mid");
        let new = dir.join("moved.mid");
        fs::write(&old, b"metadata-only scan fixture").unwrap();
        let meta = fs::metadata(&old).unwrap();
        let mut a = record(900_000);
        a.source = Some(old.clone());
        a.source_bytes = meta.len();
        a.source_modified = modified(meta.modified().ok());
        let folder = dir.join("records");
        commit(&folder, &a).unwrap();
        fs::rename(&old, &new).unwrap();
        let mut b = a.clone();
        b.source = Some(new);
        b.score = 800_000;
        commit(&folder, &b).unwrap();
        let loaded = Records::load_folder(Some(folder));
        let catalog = tangent_core::midi_library::Catalog::scan(&dir, false).unwrap();
        assert_eq!(
            loaded.for_entry(&catalog.entries[0]).unwrap().score,
            900_000
        );
        fs::remove_dir_all(dir).unwrap();
    }
}
