//! Context-free, equal-tempered chord recognition. MIDI supplies pitch, not spelling
//! or harmonic intent, so candidates are ranked hypotheses rather than probabilities.

pub mod live;
pub mod training;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Spelling {
    #[default]
    Sharps,
    Flats,
}

pub fn note_name(pitch: u8, spelling: Spelling) -> &'static str {
    let names = match spelling {
        Spelling::Sharps => [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ],
        Spelling::Flats => [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ],
    };
    names[(pitch % 12) as usize]
}

#[derive(Clone, Copy, Debug)]
pub struct RecognitionOptions {
    pub spelling: Spelling,
    /// Permit omission of the perfect fifth when the third and other tones remain.
    pub allow_missing_fifth: bool,
    /// Opt in to inferred roots; often ambiguous without a key or progression.
    pub allow_rootless: bool,
}

impl Default for RecognitionOptions {
    fn default() -> Self {
        Self {
            spelling: Spelling::Sharps,
            allow_missing_fifth: true,
            allow_rootless: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChordCandidate {
    pub symbol: String,
    /// Pitch classes, C=0 through B=11.
    pub root: u8,
    pub bass: u8,
    pub quality: &'static str,
    /// Present intervals relative to the inferred root, sorted and deduplicated.
    pub intervals: Vec<u8>,
    /// Missing template intervals, relative to root (0=root, 7=perfect fifth).
    pub missing_intervals: Vec<u8>,
    /// True when the bass is outside the chord template, e.g. C/D.
    pub non_chord_bass: bool,
    /// Heuristic ordering score, NOT a confidence percentage.
    pub score: i16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChordAnalysis {
    /// Sorted unique MIDI notes; invalid bytes above 127 are ignored.
    pub notes: Vec<u8>,
    pub pitch_classes: Vec<u8>,
    /// Lowest actual MIDI note, preserving register information.
    pub bass_note: Option<u8>,
    pub candidates: Vec<ChordCandidate>,
}

struct Template {
    suffix: &'static str,
    intervals: &'static [u8],
}

macro_rules! templates {
    ($($suffix:literal => [$($interval:literal),*]),* $(,)?) => {
        &[$(Template { suffix: $suffix, intervals: &[$($interval),*] }),*]
    };
}

const TEMPLATES: &[Template] = templates![
    "" => [0,4,7], "m" => [0,3,7], "dim" => [0,3,6], "aug" => [0,4,8],
    "sus2" => [0,2,7], "sus4" => [0,5,7], "5" => [0,7],
    "6" => [0,4,7,9], "m6" => [0,3,7,9],
    "7" => [0,4,7,10], "maj7" => [0,4,7,11], "m7" => [0,3,7,10],
    "m(maj7)" => [0,3,7,11], "dim7" => [0,3,6,9], "m7b5" => [0,3,6,10],
    "aug7" => [0,4,8,10], "augmaj7" => [0,4,8,11],
    "7sus2" => [0,2,7,10], "7sus4" => [0,5,7,10],
    "add9" => [0,2,4,7], "madd9" => [0,2,3,7],
    "add11" => [0,4,5,7], "madd11" => [0,3,5,7],
    "6/9" => [0,2,4,7,9], "m6/9" => [0,2,3,7,9],
    "9" => [0,2,4,7,10], "maj9" => [0,2,4,7,11], "m9" => [0,2,3,7,10],
    "11" => [0,2,4,5,7,10], "maj11" => [0,2,4,5,7,11], "m11" => [0,2,3,5,7,10],
    "13" => [0,2,4,5,7,9,10], "maj13" => [0,2,4,5,7,9,11], "m13" => [0,2,3,5,7,9,10],
    "7b5" => [0,4,6,10], "7#5" => [0,4,8,10],
    "7b9" => [0,1,4,7,10], "7#9" => [0,3,4,7,10],
    "7#11" => [0,4,6,7,10], "7b13" => [0,4,7,8,10],
    "9sus4" => [0,2,5,7,10], "13(no11)" => [0,2,4,7,9,10],
];

/// Recognize any set/voicing of MIDI pitches. Octave doubles and input order do
/// not affect harmony; the lowest pitch breaks otherwise equivalent ties.
/// Empty/single-note/unrecognized sets return no candidates, never a forced name.
/// All exact interpretations precede interpretations requiring omitted tones.
pub fn recognize(midi_notes: &[u8], options: RecognitionOptions) -> ChordAnalysis {
    let mut notes: Vec<_> = midi_notes.iter().copied().filter(|n| *n < 128).collect();
    notes.sort_unstable();
    notes.dedup();
    let bass_note = notes.first().copied();
    let mut pitch_classes: Vec<_> = notes.iter().map(|n| n % 12).collect();
    pitch_classes.sort_unstable();
    pitch_classes.dedup();
    let mut candidates = Vec::new();
    if pitch_classes.len() >= 2 {
        let bass = bass_note.unwrap() % 12;
        for root in 0..12 {
            let mut intervals: Vec<_> =
                pitch_classes.iter().map(|p| (p + 12 - root) % 12).collect();
            intervals.sort_unstable();
            for template in TEMPLATES {
                let missing: Vec<_> = template
                    .intervals
                    .iter()
                    .copied()
                    .filter(|i| !intervals.contains(i))
                    .collect();
                let extra: Vec<_> = intervals
                    .iter()
                    .copied()
                    .filter(|i| !template.intervals.contains(i))
                    .collect();
                let non_chord_bass = extra.len() == 1 && extra[0] == (bass + 12 - root) % 12;
                if !extra.is_empty() && !non_chord_bass {
                    continue;
                }
                if missing.iter().any(|i| {
                    !((*i == 7 && options.allow_missing_fifth)
                        || (*i == 0 && options.allow_rootless))
                }) {
                    continue;
                }
                // Do not extrapolate a chord from an isolated third or a dyad
                // with an unrelated bass. Rootless readings need 3 chord tones.
                let chord_tones = intervals.len() - extra.len();
                if chord_tones < 2 || (!missing.is_empty() && chord_tones < 3) {
                    continue;
                }
                let score = 1000 - missing.len() as i16 * 150 - if non_chord_bass { 80 } else { 0 }
                    + if root == bass { 20 } else { 0 };
                let mut symbol =
                    format!("{}{}", note_name(root, options.spelling), template.suffix);
                if root != bass {
                    symbol.push_str(&format!("/{}", note_name(bass, options.spelling)));
                }
                if !missing.is_empty() {
                    let omissions: Vec<_> = missing
                        .iter()
                        .map(|i| if *i == 0 { "no root" } else { "no5" })
                        .collect();
                    symbol.push_str(&format!(" ({})", omissions.join(", ")));
                }
                candidates.push(ChordCandidate {
                    symbol,
                    root,
                    bass,
                    quality: template.suffix,
                    intervals: intervals.clone(),
                    missing_intervals: missing,
                    non_chord_bass,
                    score,
                });
            }
        }
    }
    candidates.sort_by(|a, b| b.score.cmp(&a.score).then(a.root.cmp(&b.root)));
    ChordAnalysis {
        notes,
        pitch_classes,
        bass_note,
        candidates,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn analyze(notes: &[u8]) -> ChordAnalysis {
        recognize(notes, RecognitionOptions::default())
    }
    #[test]
    fn triads_inversions_doubles_and_input_order() {
        assert_eq!(analyze(&[72, 67, 48, 64, 60, 64]).candidates[0].symbol, "C");
        assert_eq!(analyze(&[64, 67, 72]).candidates[0].symbol, "C/E");
        assert_eq!(analyze(&[57, 60, 64]).candidates[0].symbol, "Am");
    }
    #[test]
    fn ambiguity_and_bass_ranking() {
        let a = analyze(&[48, 64, 67, 69]);
        assert_eq!(a.candidates[0].symbol, "C6");
        assert!(a.candidates.iter().any(|c| c.symbol == "Am7/C"));
        assert_eq!(analyze(&[45, 60, 64, 67]).candidates[0].symbol, "Am7");
        assert_eq!(
            analyze(&[60, 63, 66, 69])
                .candidates
                .iter()
                .filter(|c| c.missing_intervals.is_empty() && !c.non_chord_bass)
                .count(),
            4
        );
    }
    #[test]
    fn families_and_transpositions() {
        for (notes, name) in [
            (vec![60, 64, 67, 71], "Cmaj7"),
            (vec![60, 63, 66, 70], "Cm7b5"),
            (vec![60, 64, 67, 70, 74], "C9"),
            (vec![60, 64, 67, 74], "Cadd9"),
            (vec![60, 64, 67, 70, 73], "C7b9"),
            (vec![60, 67], "C5"),
            (vec![60, 64, 67, 70, 74, 77, 81], "C13"),
        ] {
            assert!(
                analyze(&notes).candidates.iter().any(|c| c.symbol == name),
                "{name}"
            );
        }
        for root in 0..12 {
            let a = analyze(&[48 + root, 52 + root, 55 + root]);
            assert_eq!(a.candidates[0].root, root);
            assert_eq!(a.candidates[0].quality, "");
        }
    }
    #[test]
    fn omissions_and_slash_bass_are_explicit() {
        let a = analyze(&[60, 64, 70]);
        let c = a
            .candidates
            .iter()
            .find(|c| c.root == 0 && c.quality == "7")
            .unwrap();
        assert_eq!(c.missing_intervals, [7]);
        assert_eq!(c.symbol, "C7 (no5)");
        assert!(
            recognize(
                &[60, 64, 70],
                RecognitionOptions {
                    allow_missing_fifth: false,
                    ..Default::default()
                }
            )
            .candidates
            .is_empty()
        );
        let a = recognize(
            &[64, 70, 74],
            RecognitionOptions {
                allow_rootless: true,
                ..Default::default()
            },
        );
        assert!(
            a.candidates
                .iter()
                .any(|c| c.root == 0 && c.quality == "9" && c.missing_intervals == [0, 7])
        );
        assert!(
            analyze(&[50, 60, 64, 67])
                .candidates
                .iter()
                .any(|c| c.symbol == "C/D" && c.non_chord_bass)
        );
    }
    #[test]
    fn empty_single_unknown_invalid_and_spelling() {
        for notes in [
            vec![],
            vec![60],
            vec![60, 72],
            vec![60, 61, 62],
            vec![128, 255],
        ] {
            assert!(analyze(&notes).candidates.is_empty());
        }
        assert_eq!(
            recognize(
                &[61, 65, 68],
                RecognitionOptions {
                    spelling: Spelling::Flats,
                    ..Default::default()
                }
            )
            .candidates[0]
                .symbol,
            "Db"
        );
    }
    #[test]
    fn every_pitch_set_is_safe_and_deterministic() {
        for mask in 0u16..4096 {
            let notes: Vec<_> = (0..12)
                .filter(|n| mask & (1 << n) != 0)
                .map(|n| 60 + n)
                .collect();
            let a = analyze(&notes);
            assert!(a.candidates.windows(2).all(|p| p[0].score >= p[1].score));
            assert_eq!(a, analyze(&notes));
        }
    }
}
