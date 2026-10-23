# Live chord recognition

In **Free Play**, click **Chords: Off** to turn on chord names. Play notes on the
selected MIDI input (or the on-screen/PC keyboard). The name above the keyboard
updates on note presses, releases, and sustain changes. Click **Chords: On** to hide
it. This switch belongs to the current Free Play session and defaults to off.

The first line is the best-ranked interpretation. `Also:` shows up to two
alternatives. For example, C–E–G–A can be **C6** or **Am7/C**. Slash names preserve
the actual bass, so E–G–C is **C/E**. Octave doubling and input order do not change
the chord. A single pitch class is displayed as a note; unsupported combinations
show `No chord match` with their pitch classes. No name is forced onto a cluster.

Sustain (CC64) retains released notes until pedal release, independently on each
MIDI channel. All Sound Off (CC120), Reset All Controllers (CC121), and All Notes
Off (CC123) are handled. Percussion channel 10 is excluded. The analysis includes
the full MIDI range, even notes outside the visible keyboard. Recording preview
playback does not enter the live analysis. Sostenuto, half-pedaling, acoustic decay,
and audio/microphone recognition are not modeled.

## Reusable API

```rust
use neothesia_core::chords::{recognize, RecognitionOptions, Spelling};

let result = recognize(&[64, 67, 72], RecognitionOptions::default());
assert_eq!(result.candidates[0].symbol, "C/E");
assert_eq!(result.bass_note, Some(64));

let options = RecognitionOptions {
    spelling: Spelling::Flats,
    allow_missing_fifth: true,
    allow_rootless: true,
};
let hypotheses = recognize(&[64, 70, 74], options);
// Includes C9/E (no root, no5), alongside other interpretations.
```

Input is a slice of MIDI note numbers (0–127), in any order. Values outside the
MIDI range are ignored. No key, timestamp, velocity, or hand assignment is required.
Keep notes active using `chords::live::LiveNotes` when consuming a MIDI stream;
pass NoteOn (including zero velocity), NoteOff, and controller events to it.

`ChordAnalysis` contains sorted unique MIDI notes, pitch classes, lowest MIDI note,
and every matching candidate. Each `ChordCandidate` includes the symbol, root and
bass pitch classes, quality, observed intervals, missing intervals, whether the
bass is outside the template, and a deterministic ranking score. Intervals are
pitch classes relative to the candidate root, not octave-specific scale degrees.
`score` is a heuristic ordering value, **not a confidence probability**.

Recognition currently covers major/minor/diminished/augmented triads, power and
suspended chords, sixths, sevenths, added ninths/elevenths, 6/9, ninths,
elevenths, thirteenths, and several altered dominants. Candidates match templates
exactly, except for explicitly labeled missing perfect fifths, opt-in missing roots,
and a single additional bass pitch class. Missing-tone interpretations need at least
three observed chord tones. Other omitted extensions are not automatically inferred.
Rootless recognition defaults to off to avoid flooding ordinary chords with
speculative extended names. Extend `TEMPLATES` to add further chord vocabulary.

Exact template matches rank ahead of additional-bass interpretations, then
incomplete voicings; bass-root agreement breaks ties, followed by pitch-class and
stable template ordering. Equivalent symbols such as aug7 and 7#5 can both appear.
The UI uses default options (sharp names, missing fifth permitted, rootless off).

## Limits

A MIDI note set does not uniquely specify harmonic function or spelling. C6 and
Am7 can contain identical pitches; C# and Db are the same MIDI pitch. Correct
contextual naming may require a key, preceding chords, melody, and notation.
The API offers a sharp/flat preference but does not infer a key or spelling.
A template catalog cannot name every arbitrary cluster or polychord.

The live view analyzes the current sounding-note set immediately. Rolled chords
pass through intermediate names; long sustain can mix successive harmonies.
There is no chord-boundary detection, arpeggio accumulation, or musical-context
model in this first implementation.

## Verification

The dependency-free engine and note-state tests can run without graphics/audio:

```sh
rustc --edition=2024 --test neothesia-core/src/chords/mod.rs -o /tmp/chord-tests
/tmp/chord-tests
```

Tests cover inversion and octave handling, ambiguous sixth/seventh interpretations,
transposition, extended/altered chords, omitted notes, spelling, unknown input,
all 4096 pitch-class sets, note releases, velocity-zero events, channel isolation,
sustain/retrigger, and controller resets. CI runs this suite separately from the
existing application builds.

Physical/UI smoke test (requires a build from this branch):

1. Open Free Play with the piano selected. Enable **Chords**; expect `Play a chord`.
2. Play C4–E4–G4; expect **C** first. Play E4–G4–C5; expect **C/E** first.
3. Play C3–E4–G4–A4; expect **C6**, with **Am7/C** among alternatives.
4. Release G4 from a C major triad; the displayed set must change immediately.
   Release every key with pedal up; expect `Play a chord`.
5. Hold sustain, play C major, release keys; expect the chord to remain.
   Release sustain; expect `Play a chord`. Repeat while keeping E held; expect `Note: E`.
6. Disable the display, play a different chord, then enable it while holding notes;
   expect the current chord immediately. Check that recording/preview still works
   and preview notes alone do not appear in the live chord display.

Physical MIDI and visual GUI verification must be performed on a desktop with a piano.

## Chord training

In Free Play, enable **Training: Off/On** next to the chord-display switch.
The current task shows its chord name, suggested notes with octave numbers
(C4 is middle C), and progress through 20 exercises. Gold markers just above
the keyboard identify the suggested voicing; they turn green when correct.
The notes are displayed in root position, but answers accept any octave,
inversion, or octave doubling. Every target pitch class must be present and
no additional pitch classes may sound. Recognition ranking/alternative names
do not affect answer validation.

Hold the correct set for 300 ms. The view displays **Correct!**, then waits until
all keys and sustained notes are released before moving to the next task.
If keys are already sounding when starting/restarting, release them first.
Wrong, incomplete, or extra notes reset the hold timer without skipping tasks.
After the last chord, the completion screen remains until you restart or exit.
**Restart** begins again at C; turning training off returns to ordinary Free Play.
The chord-display switch works independently. Training only evaluates live input,
not recording-preview playback; pause a preview if it is distracting.

The curated beginner sequence is C, F, G, Am, Dm, Em, D, A, E, Bm, Bb, Eb,
G7, D7, A7, E7, Cmaj7, Am7, Dm7, Fmaj7. This is a practical selection of
common chords rather than a statistical frequency ranking. Progress is kept for
the current session; there is no score history or randomization yet.

Additional physical/UI smoke test:

1. Enable Training with all notes released. Expect `Chord 1 / 20: C`,
   `C4 - E4 - G4`, and three gold keyboard markers.
2. Play C-E only, C-Eb-G, or C-E-G-A. None must advance the exercise.
3. Play E-G-C in any octave for 300 ms. Expect **Correct!** and green markers.
4. Keep a note or sustain held. The task must stay on C; release all notes/pedal
   and expect F with its new note names/markers.
5. Restart while notes sound. Expect a release prompt, followed by the first task.
6. Complete the series; expect `20 / 20` and a restart option. Toggle Training off
   and verify ordinary Free Play and the independent Chords switch still work.

Six automated training tests cover hold timing, accepted inversions/doubles,
wrong/extra/incomplete notes, initial release, sustained-note release gating,
full-series completion/restart, and displayed note spelling.
