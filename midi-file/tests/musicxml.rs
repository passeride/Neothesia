use midi_file::{MidiFile, musicxml};
use std::io::{Cursor, Write};
const STUDY: &str = include_str!("fixtures/piano-study.musicxml");
fn score(body: &str) -> String {
    format!(
        r#"<score-partwise version="4.0"><part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list><part id="P1"><measure number="1"><attributes><divisions>3</divisions></attributes>{body}</measure></part></score-partwise>"#
    )
}
fn note(step: &str, duration: u32) -> String {
    format!(
        "<note><pitch><step>{step}</step><octave>4</octave></pitch><duration>{duration}</duration></note>"
    )
}
#[test]
fn imports_piano_spelling_pickup_chord_tie_tempo_and_rests() {
    let f = musicxml::import(STUDY, "study".into()).unwrap();
    let s = f.score.as_ref().unwrap();
    assert_eq!(s.title.as_deref(), Some("Tangent Piano Study"));
    assert_eq!(s.notes.len(), 8);
    assert_eq!(s.notes[2].pitch, Some(('C', 1, 5)));
    assert_eq!(s.notes[2].dots, 1);
    assert!(s.notes[5].tie_stop);
    assert_eq!(
        s.track_names,
        ["Tempo", "P1 · staff 1 · voice 1", "P1 · staff 2 · voice 2"]
    );
    let notes = &f.tracks[1].notes;
    assert_eq!(notes.len(), 3); // tied continuation is no second attack
    let tied = notes.iter().find(|n| n.note == 73).unwrap();
    assert!((tied.start.as_secs_f64() - 0.5).abs() < 1e-6);
    assert!((tied.end.as_secs_f64() - 2.666667).abs() < 1e-6);
    assert_eq!(
        f.measures.iter().map(|m| m.as_millis()).collect::<Vec<_>>(),
        [0, 500, 2000]
    );
}
#[test]
fn exact_tuplets_and_repeated_pitch_release_before_attack() {
    let f = musicxml::import(
        &score(&(note("C", 1) + &note("C", 1) + &note("C", 1))),
        "triplet".into(),
    )
    .unwrap();
    assert_eq!(f.tracks[1].notes.len(), 3);
    assert_eq!(f.tracks[1].notes[0].end, f.tracks[1].notes[1].start);
    assert_eq!(f.tracks[1].notes[2].end.as_millis(), 500);
}
#[test]
fn rejects_structures_that_would_change_the_playback_chart() {
    for element in [
        "repeat",
        "ending",
        "grace",
        "unpitched",
        "transpose",
        "octave-shift",
        "ornaments",
        "pedal",
        "arpeggiate",
        "fermata",
    ] {
        let xml = score(&(note("C", 3) + &format!("<{element}/>")));
        assert!(
            musicxml::import(&xml, "bad".into())
                .unwrap_err()
                .contains(element)
        );
    }
    for body in [
        "<backup><duration>3</duration></backup>",
        "<note><rest/><duration>0</duration></note>",
        "<sound tempo=\"NaN\"/>",
        "<sound dacapo=\"yes\"/>",
    ] {
        assert!(musicxml::import(&score(body), "bad".into()).is_err());
    }
    assert!(musicxml::import(&score(&note("C", 13)), "overfull".into()).is_err());
    assert!(musicxml::import("<score-timewise/>", "timewise".into()).is_err());
    assert!(musicxml::import("<score-partwise>", "malformed".into()).is_err());
    assert!(musicxml::import(&score(&note("CC", 3)), "invalid pitch".into()).is_err());
}
#[test]
fn rejects_broken_ties_and_conflicting_tempo() {
    assert!(musicxml::import(&STUDY.replace("<tie type=\"start\"/>", ""), "tie".into()).is_err());
    assert!(
        musicxml::import(
            &score("<sound tempo=\"60\"/><sound tempo=\"90\"/>"),
            "tempo".into()
        )
        .is_err()
    );
}
#[test]
fn external_doctype_is_inert_but_entities_are_rejected() {
    assert!(musicxml::import(STUDY, "doctype".into()).is_ok());
    let xml = format!(
        "<!DOCTYPE score-partwise [<!ENTITY x SYSTEM 'file:///etc/passwd'>]>{}",
        score(&note("C", 3))
    );
    assert!(musicxml::import(&xml, "entity".into()).is_err());
}
fn archive(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in entries {
        zip.start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(data.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}
fn load_bytes(bytes: &[u8], ext: &str) -> Result<MidiFile, String> {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "tangent-xml-{}-{}.{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        ext
    ));
    std::fs::write(&path, bytes).unwrap();
    let r = MidiFile::new(&path);
    std::fs::remove_file(path).unwrap();
    r
}
#[test]
fn opens_all_extensions_and_container_root_not_filename_guess() {
    for ext in ["musicxml", "xml", "MUSICXML"] {
        assert!(load_bytes(STUDY.as_bytes(), ext).unwrap().score.is_some());
    }
    let zip = archive(&[
        (
            "META-INF/container.xml",
            r#"<container><rootfiles><rootfile full-path="scores/piano.musicxml"/></rootfiles></container>"#,
        ),
        ("scores/piano.musicxml", STUDY),
    ]);
    assert_eq!(load_bytes(&zip, "mxl").unwrap().tracks[1].notes.len(), 3);
}
#[test]
fn mxl_rejects_missing_root_traversal_entities_and_oversized_input() {
    for entries in [
        vec![("score.xml", STUDY)],
        vec![("../score.xml", STUDY)],
        vec![(
            "META-INF/container.xml",
            "<!DOCTYPE container [<!ENTITY x 'bad'>]><container/>",
        )],
        vec![(
            "META-INF/container.xml",
            "<container><rootfile full-path='../score.xml'/></container>",
        )],
    ] {
        assert!(load_bytes(&archive(&entries), "mxl").is_err());
    }
    assert!(load_bytes(&vec![b' '; 16 * 1024 * 1024 + 1], "musicxml").is_err());
}
