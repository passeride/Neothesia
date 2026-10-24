//! Bounded MusicXML piano import. Written notes are retained alongside playback.
use crate::MidiFile;
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
use roxmltree::{Document, Node};
use std::{collections::BTreeMap, io::Read, path::Path, sync::Arc};

const PPQ: u16 = 15120;
const LIMIT: u64 = 16 * 1024 * 1024;
const MAX_TICKS: u64 = PPQ as u64 * 4 * 60 * 24;

#[derive(Clone, Debug)]
pub struct WrittenNote {
    pub part: String,
    pub measure: String,
    pub voice: String,
    pub staff: u8,
    pub tick: u64,
    pub duration: u64,
    /// Diatonic spelling is independent of MIDI pitch (e.g. C sharp / D flat).
    pub pitch: Option<(char, i8, i8)>,
    pub note_type: Option<String>,
    pub dots: usize,
    pub tie_start: bool,
    pub tie_stop: bool,
}

#[derive(Clone, Debug)]
pub struct Score {
    pub title: Option<String>,
    pub notes: Vec<WrittenNote>,
    pub track_names: Vec<String>,
    /// Original XML retains clefs, key signatures, layout and source provenance.
    /// The current Staff renderer still uses generated MIDI notation.
    pub xml: Arc<str>,
}

type N<'a, 'input> = Node<'a, 'input>;
fn child<'a, 'i>(n: N<'a, 'i>, tag: &str) -> Option<N<'a, 'i>> {
    n.children().find(|n| n.has_tag_name(tag))
}
fn value<'a>(n: N<'a, '_>, tag: &str) -> Option<&'a str> {
    child(n, tag).and_then(|n| n.text())
}
fn number(n: N<'_, '_>, tag: &str) -> Result<i64, String> {
    value(n, tag)
        .ok_or_else(|| format!("Missing {tag}"))?
        .parse()
        .map_err(|_| format!("Invalid integer {tag}"))
}
fn ticks(n: N<'_, '_>, divisions: i64) -> Result<u64, String> {
    let d = number(n, "duration")?;
    if d <= 0 || divisions <= 0 {
        return Err("Duration/divisions must be positive".into());
    }
    let t = d.checked_mul(i64::from(PPQ)).ok_or("Duration overflow")?;
    if t % divisions != 0 {
        return Err("Rhythm cannot be represented exactly by this importer".into());
    }
    let ticks = u64::try_from(t / divisions).map_err(|_| "Duration overflow")?;
    if ticks > MAX_TICKS {
        return Err("MusicXML duration exceeds supported score length".into());
    }
    Ok(ticks)
}
fn bounded(mut r: impl Read) -> Result<String, String> {
    let mut bytes = Vec::new();
    r.by_ref()
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > LIMIT {
        return Err("MusicXML exceeds 16 MiB limit".into());
    }
    String::from_utf8(bytes).map_err(|_| "MusicXML must be UTF-8".into())
}
fn parse(xml: &str) -> Result<Document<'_>, String> {
    if xml.contains("<!ENTITY")
        || xml.find("<!DOCTYPE").is_some_and(|at| {
            xml[at..]
                .split('>')
                .next()
                .unwrap_or_default()
                .contains('[')
        })
    {
        return Err("MusicXML entity declarations/internal DTD subsets are unsupported".into());
    }
    // roxmltree never resolves external resources.
    // Common external DOCTYPE declarations are accepted, internal subsets are not.
    Document::parse_with_options(
        xml,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            nodes_limit: 500_000,
            ..Default::default()
        },
    )
    .map_err(|e| format!("Invalid MusicXML: {e}"))
}
fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':'])
        && !path.starts_with('/')
        && path
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != "..")
}

pub fn load(path: &Path, name: String) -> Result<MidiFile, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > LIMIT * 2 {
        return Err("Score file exceeds 32 MiB limit".into());
    }
    let xml = if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("mxl"))
    {
        let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("Invalid MXL: {e}"))?;
        if zip.len() > 1024 {
            return Err("MXL has too many entries".into());
        }
        let mut total = 0u64;
        let mut names = std::collections::BTreeSet::new();
        for i in 0..zip.len() {
            let e = zip.by_index(i).map_err(|e| e.to_string())?;
            let raw_name = e.name().map_err(|e| e.to_string())?;
            let name = raw_name.trim_end_matches('/');
            if !safe_path(name) || !names.insert(raw_name.into_owned()) {
                return Err("Unsafe or duplicate MXL entry".into());
            }
            total = total.checked_add(e.size()).ok_or("MXL size overflow")?;
            if total > LIMIT * 2 {
                return Err("MXL uncompressed size exceeds 32 MiB".into());
            }
        }
        let container = bounded(
            zip.by_name("META-INF/container.xml")
                .map_err(|_| "MXL has no META-INF/container.xml")?,
        )?;
        let doc = parse(&container)?;
        if !doc.root_element().has_tag_name("container") {
            return Err("Invalid MXL container root".into());
        }
        let root = doc
            .descendants()
            .find(|n| n.has_tag_name("rootfile"))
            .and_then(|n| n.attribute("full-path"))
            .ok_or("MXL has no rootfile")?;
        if !safe_path(root) {
            return Err("Unsafe MXL rootfile path".into());
        }
        bounded(zip.by_name(root).map_err(|_| "MXL rootfile is missing")?)?
    } else {
        bounded(file)?
    };
    import(&xml, name)
}

#[derive(Clone)]
struct Attack {
    start: u64,
    end: u64,
    pitch: u8,
}
type TrackKey = (String, u8, String);

pub fn import(xml: &str, name: String) -> Result<MidiFile, String> {
    // No entity declarations, including internal entity expansion. External DTD
    // names in ordinary exporter headers are inert and never fetched.
    if xml.contains("<!ENTITY")
        || xml.find("<!DOCTYPE").is_some_and(|at| {
            xml[at..]
                .split('>')
                .next()
                .unwrap_or_default()
                .contains('[')
        })
    {
        return Err("MusicXML entity declarations/internal DTD subsets are unsupported".into());
    }
    let doc = parse(xml)?;
    let root = doc.root_element();
    if !root.has_tag_name("score-partwise") {
        return Err("Only score-partwise MusicXML is supported".into());
    }
    let unsupported = [
        "repeat",
        "ending",
        "grace",
        "cue",
        "unpitched",
        "transpose",
        "octave-shift",
        "ornaments",
        "arpeggiate",
        "glissando",
        "slide",
        "pedal",
        "swing",
        "scordatura",
        "fermata",
        "senza-misura",
        "measure-repeat",
        "beat-repeat",
    ];
    for n in root.descendants().filter(|n| n.is_element()) {
        if unsupported.contains(&n.tag_name().name()) {
            return Err(format!(
                "MusicXML <{}> is not supported yet; simplify/export a linear score",
                n.tag_name().name()
            ));
        }
        if n.has_tag_name("sound")
            && n.attributes().any(|a| {
                matches!(
                    a.name(),
                    "dacapo"
                        | "dalsegno"
                        | "tocoda"
                        | "fine"
                        | "segno"
                        | "coda"
                        | "damper-pedal"
                        | "soft-pedal"
                        | "sostenuto-pedal"
                )
            })
        {
            return Err("MusicXML playback jumps are not supported".into());
        }
        if n.has_tag_name("tie") && !matches!(n.attribute("type"), Some("start" | "stop")) {
            return Err("Invalid MusicXML tie type".into());
        }
        if n.has_tag_name("note")
            && (n.attribute("attack").is_some() || n.attribute("release").is_some())
        {
            return Err("MusicXML attack/release offsets are not supported".into());
        }
    }
    let mut score = Score {
        title: root
            .descendants()
            .find(|n| n.has_tag_name("work-title") || n.has_tag_name("movement-title"))
            .and_then(|n| n.text())
            .map(str::to_owned),
        notes: Vec::new(),
        track_names: vec!["Tempo".into()],
        xml: Arc::from(xml),
    };
    let mut tracks: BTreeMap<TrackKey, Vec<Attack>> = BTreeMap::new();
    let mut tempos: BTreeMap<u64, u32> = BTreeMap::from([(0, 500_000)]);
    let mut authored_tempos = BTreeMap::new();
    let mut measure_starts = Vec::new();
    let mut reference_measures: Option<Vec<(u64, u64)>> = None;
    let mut ids = std::collections::BTreeSet::new();
    for part in root.children().filter(|n| n.has_tag_name("part")) {
        let id = part.attribute("id").ok_or("Part has no id")?.to_owned();
        if !ids.insert(id.clone()) {
            return Err("Duplicate part id".into());
        }
        let mut divisions = 0;
        let mut nominal = u64::from(PPQ) * 4;
        let mut base = 0u64;
        let mut spans = Vec::new();
        let mut ties: BTreeMap<(u8, String, u8), (TrackKey, usize)> = BTreeMap::new();
        for measure in part.children().filter(|n| n.has_tag_name("measure")) {
            let label = measure.attribute("number").unwrap_or("?").to_owned();
            let mut cursor = 0u64;
            let mut extent = 0u64;
            let mut previous: Option<(u64, u64, u8, String)> = None;
            for e in measure.children().filter(|n| n.is_element()) {
                match e.tag_name().name() {
                    "attributes" => {
                        if child(e, "divisions").is_some() {
                            divisions = number(e, "divisions")?;
                            if divisions <= 0 {
                                return Err("divisions must be positive".into());
                            }
                        }
                        if let Some(time) = child(e, "time") {
                            if cursor != 0
                                || time.children().filter(|n| n.has_tag_name("beats")).count() != 1
                            {
                                return Err(
                                    "Mid-measure/additive time signatures are not supported".into(),
                                );
                            }
                            let beats = number(time, "beats")?;
                            let unit = number(time, "beat-type")?;
                            if !(1..=64).contains(&beats)
                                || !(1..=64).contains(&unit)
                                || (unit & (unit - 1)) != 0
                            {
                                return Err("Unsupported time signature".into());
                            }
                            nominal = u64::from(PPQ) * 4 * beats as u64 / unit as u64;
                        }
                    }
                    "backup" => {
                        cursor = cursor
                            .checked_sub(ticks(e, divisions)?)
                            .ok_or("Backup before measure start")?;
                        previous = None;
                    }
                    "forward" => {
                        cursor = cursor
                            .checked_add(ticks(e, divisions)?)
                            .ok_or("Position overflow")?;
                        extent = extent.max(cursor);
                        previous = None;
                    }
                    "direction" | "sound" => {
                        let at = if let Some(off) = value(e, "offset") {
                            let d: i64 = off.parse().map_err(|_| "Invalid direction offset")?;
                            let t = d.checked_mul(i64::from(PPQ)).ok_or("Offset overflow")?;
                            if divisions <= 0 || t % divisions != 0 {
                                return Err("Unsupported direction offset".into());
                            }
                            (base + cursor)
                                .checked_add_signed(t / divisions)
                                .ok_or("Direction before score start")?
                        } else {
                            base + cursor
                        };
                        let sound = if e.has_tag_name("sound") {
                            Some(e)
                        } else {
                            child(e, "sound")
                        };
                        if e.descendants().any(|n| n.has_tag_name("metronome"))
                            && sound.and_then(|s| s.attribute("tempo")).is_none()
                        {
                            return Err(
                                "Metronome requires explicit sound tempo for playback".into()
                            );
                        }
                        if let Some(tempo) = sound.and_then(|s| s.attribute("tempo")) {
                            let bpm: f64 = tempo.parse().map_err(|_| "Invalid tempo")?;
                            if !bpm.is_finite() || !(4.0..=1000.0).contains(&bpm) {
                                return Err("Tempo must be 4–1000 BPM".into());
                            }
                            let us = (60_000_000.0 / bpm).round() as u32;
                            if authored_tempos.insert(at, us).is_some_and(|old| old != us) {
                                return Err("Conflicting tempos at the same position".into());
                            }
                            tempos.insert(at, us);
                        }
                    }
                    "note" => {
                        if score.notes.len() >= 100_000 {
                            return Err("Too many MusicXML notes".into());
                        }
                        let duration = ticks(e, divisions)?;
                        let voice = value(e, "voice").unwrap_or("1").to_owned();
                        let staff = match value(e, "staff") {
                            Some(s) => s.parse::<u8>().map_err(|_| "Invalid staff")?,
                            None => 1,
                        };
                        if !(1..=2).contains(&staff) {
                            return Err("Only piano staff 1/2 supported".into());
                        }
                        let chord = child(e, "chord").is_some();
                        let start = if chord {
                            let (t, d, s, v) =
                                previous.as_ref().ok_or("Chord has no preceding note")?;
                            if duration > *d || staff != *s || voice != *v {
                                return Err("Unsupported chord voice/staff/duration".into());
                            }
                            *t
                        } else {
                            let t = cursor;
                            cursor = cursor.checked_add(duration).ok_or("Position overflow")?;
                            previous = Some((t, duration, staff, voice.clone()));
                            t
                        };
                        extent = extent.max(start + duration);
                        let pitch = if child(e, "rest").is_some() {
                            previous = None;
                            None
                        } else {
                            let p = child(e, "pitch").ok_or("Note needs pitch or rest")?;
                            if !value(p, "step").is_some_and(|s| s.len() == 1) {
                                return Err("Invalid pitch step".into());
                            }
                            let step = value(p, "step")
                                .and_then(|s| s.chars().next())
                                .ok_or("Missing pitch step")?;
                            let alter = match value(p, "alter") {
                                Some(s) => s
                                    .parse::<i8>()
                                    .map_err(|_| "Microtonal/non-integer alter unsupported")?,
                                None => 0,
                            };
                            let octave =
                                i8::try_from(number(p, "octave")?).map_err(|_| "Invalid octave")?;
                            Some((step, alter, octave))
                        };
                        let tie_start = e
                            .children()
                            .any(|n| n.has_tag_name("tie") && n.attribute("type") == Some("start"));
                        let tie_stop = e
                            .children()
                            .any(|n| n.has_tag_name("tie") && n.attribute("type") == Some("stop"));
                        if let Some((step, alter, octave)) = pitch {
                            let semitone = match step {
                                'C' => 0,
                                'D' => 2,
                                'E' => 4,
                                'F' => 5,
                                'G' => 7,
                                'A' => 9,
                                'B' => 11,
                                _ => return Err("Invalid pitch step".into()),
                            };
                            let p = 12 * (i16::from(octave) + 1) + semitone + i16::from(alter);
                            if !(0..=127).contains(&p) || !(-2..=2).contains(&alter) {
                                return Err("Pitch out of supported MIDI range".into());
                            }
                            let p = p as u8;
                            let key = (id.clone(), staff, voice.clone());
                            let tk = (staff, voice.clone(), p);
                            if tie_stop {
                                let (k, i) = ties.remove(&tk).ok_or("Tie stop has no start")?;
                                let a = &mut tracks.get_mut(&k).ok_or("Missing tie track")?[i];
                                if a.end != base + start {
                                    return Err("Non-contiguous tie".into());
                                }
                                a.end = base + start + duration;
                                if tie_start {
                                    ties.insert(tk, (k, i));
                                }
                            } else {
                                let t = tracks.entry(key.clone()).or_default();
                                t.push(Attack {
                                    start: base + start,
                                    end: base + start + duration,
                                    pitch: p,
                                });
                                if tie_start && ties.insert(tk, (key, t.len() - 1)).is_some() {
                                    return Err("Duplicate tie start".into());
                                }
                            }
                        } else if chord || tie_start || tie_stop {
                            return Err("Rest cannot be a chord/tie".into());
                        }
                        score.notes.push(WrittenNote {
                            part: id.clone(),
                            measure: label.clone(),
                            voice,
                            staff,
                            tick: base + start,
                            duration,
                            pitch,
                            note_type: value(e, "type").map(str::to_owned),
                            dots: e.children().filter(|n| n.has_tag_name("dot")).count(),
                            tie_start,
                            tie_stop,
                        });
                    }
                    "barline" | "print" => {}
                    _ => {
                        return Err(format!(
                            "Unsupported measure element <{}>",
                            e.tag_name().name()
                        ));
                    }
                }
            }
            let length = if measure.attribute("implicit") == Some("yes") {
                extent
            } else {
                nominal
            };
            if length == 0 || extent > length {
                return Err(format!("Invalid/overfull measure {label}"));
            }
            spans.push((base, length));
            base = base.checked_add(length).ok_or("Score too long")?;
            if base > MAX_TICKS {
                return Err("Score exceeds 24-hour bound".into());
            }
        }
        if !ties.is_empty() {
            return Err("Unclosed tie".into());
        }
        if let Some(reference) = &reference_measures {
            if reference != &spans {
                return Err("Parts have inconsistent measure boundaries".into());
            }
        } else {
            measure_starts = spans.iter().map(|s| s.0).collect();
            reference_measures = Some(spans);
        }
    }
    if tracks.is_empty() || tracks.len() > 15 {
        return Err("MusicXML needs 1–15 pitched part/staff/voice tracks".into());
    }
    let mut events: Vec<Vec<(u64, TrackEventKind<'static>)>> = Vec::new();
    events.push(
        tempos
            .into_iter()
            .map(|(t, v)| (t, TrackEventKind::Meta(MetaMessage::Tempo(v.into()))))
            .collect(),
    );
    for (i, ((part, staff, voice), mut attacks)) in tracks.into_iter().enumerate() {
        score
            .track_names
            .push(format!("{part} · staff {staff} · voice {voice}"));
        attacks.sort_by_key(|a| a.start);
        let mut ends = [0u64; 128];
        for a in &attacks {
            if ends[a.pitch as usize] > a.start {
                return Err("Overlapping unisons in one voice are unsupported".into());
            }
            ends[a.pitch as usize] = a.end;
        }
        let channel = if i >= 9 { i + 1 } else { i } as u8;
        let mut ev = vec![(
            0,
            TrackEventKind::Midi {
                channel: channel.into(),
                message: MidiMessage::ProgramChange { program: 0.into() },
            },
        )];
        for a in attacks {
            ev.push((
                a.start,
                TrackEventKind::Midi {
                    channel: channel.into(),
                    message: MidiMessage::NoteOn {
                        key: a.pitch.into(),
                        vel: 80.into(),
                    },
                },
            ));
            ev.push((
                a.end,
                TrackEventKind::Midi {
                    channel: channel.into(),
                    message: MidiMessage::NoteOff {
                        key: a.pitch.into(),
                        vel: 0.into(),
                    },
                },
            ));
        }
        // Releases before attacks at a shared timestamp preserve repeated notes.
        ev.sort_by_key(|(t, k)| {
            (
                *t,
                matches!(
                    k,
                    TrackEventKind::Midi {
                        message: MidiMessage::NoteOn { .. },
                        ..
                    }
                ),
            )
        });
        events.push(ev);
    }
    let mut smf = Smf::new(Header::new(Format::Parallel, Timing::Metrical(PPQ.into())));
    for ev in events {
        let mut last = 0;
        let mut track = Vec::new();
        for (t, kind) in ev {
            let delta = t - last;
            if delta > 0x0fff_ffff {
                return Err("MusicXML event gap too large".into());
            }
            track.push(TrackEvent {
                delta: (delta as u32).into(),
                kind,
            });
            last = t;
        }
        track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });
        smf.tracks.push(track);
    }
    let mut result = MidiFile::from_smf(name, &smf)?;
    result.measures = measure_starts
        .into_iter()
        .map(|t| result.tempo_track.pulses_to_duration(t))
        .collect::<Vec<_>>()
        .into();
    result.score = Some(Arc::new(score));
    Ok(result)
}
