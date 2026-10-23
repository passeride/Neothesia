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

## Live notation and single-note practice

**Notes: On** shows sounding MIDI pitches on two five-line staves, with G/treble
and F/bass clefs, quarter-note heads/stems, sharps and ledger lines. The footer
shows note names with scientific octave numbers (C4 = MIDI 60 = middle C).
The display uses international B, not Norwegian H. Staves cover C2–C6; MIDI
pitches outside that window remain in the named footer, with an outside-view
count. At most 12 sounding notes are drawn, depending on window width. Notes
are spaced horizontally for readability; this is an instantaneous pitch view,
not a scrolling score or rhythmic transcription. Sharps are used consistently;
flat/enharmonic spelling and key signatures are not inferred.

**Note practice: On** asks you to read one gold note on the staff and play it.
The answer name and target-key marker are hidden initially. **Hint: On** reveals
both; after success the name is always revealed and the target turns green.
White notes and the `Played:` footer show what you are actually playing.
Training uses the exercise's clef for the correctly played target, including
middle C written above the bass staff.

- Click the range control to cycle **Both clefs → Treble → Bass → Both clefs**.
  Treble covers C4–C6, bass covers C2–C4. Both alternates clefs, ten tasks each.
- Each round has **20 shuffled tasks** with no consecutive repeated MIDI pitch.
  Restart creates a fresh sequence. Range/accidental changes restart the round.
- Start with **Sharps: Off** (white keys); enable it to add black keys with explicit
  sharp signs. There are no flat-reading exercises in this first version.
- Play exactly the shown MIDI pitch, in the correct octave, without extra notes,
  and hold it for 300 ms. This differs from chord training's octave-independent
  matching. Wrong notes and extras reset the hold timer.
- Release keys and sustained notes after success to advance. Starting/restarting
  while notes are sounding first asks you to release them. Completion remains
  visible until restart or exit. Progress is not saved across sessions.
- Chord training and note practice are mutually exclusive. Switching either on
  ends the other exercise mode. Live Chords/Notes displays remain independent.
  Recorded preview playback never answers an exercise.

The controls wrap on narrower windows. The staff panel scales to available
height; if space is too small it asks you to enlarge the window rather than draw
misleading clipped notation. Pause recording preview if its sound is distracting.

Additional physical/UI smoke test:

1. Enable Notes, play C3 and E4. Expect C3 on the bass staff, E4 on the bottom
   treble line, and `Played: C3 E4`. Release keys/pedal and expect an empty staff.
2. Play C4, F#4 and C2 separately. Check middle C on its ledger line below the
   treble staff, F# in the first treble space with a sharp sign, and low C on the
   second ledger line below the bass staff.
3. Enable Note practice. Expect a gold target, clef/progress, no target name and
   no target-key marker with Hint off. Hint on must reveal name/octave and marker.
4. Try the right letter in the wrong octave, or the right note plus another key.
   Neither may pass. The exact pitch held for 300 ms must turn green and reveal
   its name; it must wait for release before advancing.
5. Check Treble, Bass and Both, natural-only and sharps, restart and completion.
   Middle C in a bass exercise must remain on the bass ledger line when matched.
6. Start note practice while holding keys/pedal, then switch to chord training
   and back. Check release prompts, exclusive exercise modes and the independent
   live displays. Resize and check that controls/staves remain usable.

Four additional automated tests verify reference staff positions/ledger lines,
sharp spelling, exact pitch/extra-note rejection and hold reset, selected ranges,
balanced mixed rounds, deterministic seeded shuffling with no immediate repeats,
release gating, full-round completion and restart. The notation geometry was
also visually inspected with the bundled Leland music font; desktop GUI and
physical MIDI verification remain pending.

## Staff size

Use the **− / Staff 100% / +** control beside the note-display/practice controls
when either is enabled. Each click changes the notation size by 25 percentage
points, from 50% to 250%. It scales staff spacing, notes, stems, clefs, sharps and
ledger lines together, for both live notation and note practice. Changing size
does not restart an exercise. Size is retained while switching these views in
the same Free Play session; a new session starts at 100%.

The panel grows to use available vertical space without covering the keyboard
or chord display. If the requested size does not fit, it uses the largest size
that fits and states the effective percentage in its heading. Enlarge/maximize
the window for more room. Fewer simultaneous notes are drawn when zoomed in;
all sounding note names remain in the footer. Heading/feedback font growth is
limited independently to preserve readability.

Desktop check: try 100%, 150%, and 250% in live Notes and Note practice; verify
that staff spacing and symbols enlarge together, notes stay on their correct
lines, and progress survives size changes. Resize the window at high zoom and
check the fitted-size indication and that notes stay above the keyboard.

## Optional reference sound / ear practice

Both **Training** and **Note practice** offer **Hear task: Off/On**. It defaults
to Off and applies to both trainers for the current Free Play session. With it
On, each new unanswered task plays once through the selected output: the exact
note/octave in note practice, or the displayed chord voicing played together.
**Replay task** plays that same task again without changing progress. Playback
lasts 800 ms at MIDI velocity 80. It begins only after the initial release prompt
has cleared. Finished rounds do not play another task. Turning sound off,
restarting, changing exercise modes, or leaving Free Play cancels active reference
notes; replays stop the preceding reference first.

Select the built-in synth (with a SoundFont) or a MIDI output in the menu.
`Hear task: No out` means No Output is currently selected. External MIDI devices
must receive channel 16: that channel is used for reference piano notes, with
sustain off, separately from the usual channel-1 live echo. Avoid using channel
16 for simultaneous live input while reference playback is enabled. Volume uses
the selected output's existing controls. Reference messages go directly to
output, not to input state, recordings, keyboard markers, or answer evaluation.
A hardware MIDI-thru loop that feeds output back as input must be disabled for
correct exercise validation.

For ear practice, leave note-reading hints off and listen/replay before answering.
Chord training still displays the chord name/notes; this option does not add an
audio-only quiz or hide its existing visual hints.

Physical check: with the option Off, advance tasks and verify no reference sound.
Enable it and expect one short example on the current unanswered task, then one
per subsequent task. Replay several times; progress must not move and no notes
may remain sounding. Check note and chord modes, restart, mode switching, sound
off, and leaving Free Play during playback. Repeat through the synth and the
intended MIDI output. Only your played input should answer tasks or be recorded.
Two automated reference-playback tests verify note/chord deadlines, one-time
NoteOff, replay cancellation, deduplication and invalid-note filtering.
