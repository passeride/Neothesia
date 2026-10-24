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
use tangent_core::chords::{recognize, RecognitionOptions, Spelling};

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
rustc --edition=2024 --test tangent-core/src/chords/mod.rs -o /tmp/chord-tests
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
  Treble covers C4–C6, bass covers C2–C4. Both alternates clefs continuously.
- Practice continues indefinitely with **no score, task counter, or ending**.
  Pitch pools reshuffle as they are exhausted, with no consecutive repeated MIDI
  pitch. Restart or range/accidental changes start a fresh continuous sequence.
- Start with **Sharps: Off** (white keys); enable it to add black keys with explicit
  sharp signs. There are no flat-reading exercises in this first version.
- Play exactly the shown MIDI pitch, in the correct octave, without extra notes,
  and hold it for 300 ms. This differs from chord training's octave-independent
  matching. Wrong notes and extras reset the hold timer.
- Release keys and sustained notes after success to advance. Starting/restarting
  while notes are sounding first asks you to release them. Turn Note practice
  off whenever you want to stop; there is no completion screen or saved score.
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
3. Enable Note practice. Expect a gold target and clef, no progress counter, no target name and
   no target-key marker with Hint off. Hint on must reveal name/octave and marker.
4. Try the right letter in the wrong octave, or the right note plus another key.
   Neither may pass. The exact pitch held for 300 ms must turn green and reveal
   its name; it must wait for release before advancing.
5. Check Treble, Bass and Both, natural-only and sharps, and restart. Continue
   beyond 20 answers and verify there is no completion screen or counter.
   Middle C in a bass exercise must remain on the bass ledger line when matched.
6. Start note practice while holding keys/pedal, then switch to chord training
   and back. Check release prompts, exclusive exercise modes and the independent
   live displays. Resize and check that controls/staves remain usable.

Four additional automated tests verify reference staff positions/ledger lines,
sharp spelling, exact pitch/extra-note rejection and hold reset, selected ranges,
alternating mixed clefs, deterministic seeded shuffling with no immediate repeats,
release gating, and 1,000 continuous answers with bounded memory. The notation geometry was
also visually inspected with the bundled Leland music font; desktop GUI and
physical MIDI verification remain pending.

## Staff size

Use the **− / Staff % / +** control beside the note-display/practice controls
when either is enabled. Each click changes the notation size by 25 percentage
points, from 50% to 250%. It scales staff spacing, notes, stems, clefs, sharps and
ledger lines together, for both live notation and note practice. Changing size
does not restart an exercise. Size is retained while switching these views in
Settings and both notation views; the saved default starts at 125%.

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
to Off and applies to both trainers. The choice is saved in Settings. With it
On, each new unanswered task plays once through the selected output: the exact
note/octave in note practice, or the displayed chord voicing played together.
**Replay task** plays that same task again without changing progress. Playback
lasts 800 ms at MIDI velocity 80. It begins only after the initial release prompt
has cleared. Note practice plays every new target indefinitely; completed chord
sequences do not play another task. Turning sound off,
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

## Stacked chord notation

Chord **Training** automatically shows staff notation below its task card,
even with the ordinary Notes display off. The gold target notes are stacked at
one horizontal position on each staff, like a written piano chord; they turn
green when the answer is correct. Your sounding notes appear as a separate white
stack beside the target. Suggested notes below middle C use the bass staff; the
others use treble. The existing suggested voicing, answer matching, keyboard
markers and reference sound are unchanged.

Each stack has a shared stem per staff. Adjacent noteheads are offset to opposite
sides of the stem to avoid overlap, and close accidentals use separate columns.
Chord tasks use their existing sharp/flat preference, including Bb and Eb.
The staff-size controls also work when only chord training is active. At narrow
width/high zoom the target is retained while played notes that do not fit remain
listed in the footer. The same C2–C6 notation range applies.

**Only chord training uses stacked notation.** Single-note practice and ordinary
live Notes retain the existing horizontal spacing and behavior. Note-reading
hints, continuous generation and optional reference playback are unaffected.

Desktop check: enable chord training with Notes off and verify C, Cmaj7 and Bb
show stacked target notes with correct ledger lines/flat signs. Play a chord and
an inversion; check the separate white stack and green success state. Test close
notes and staff zoom. Turn chord training off and return to Note practice and live
Notes; verify their original layout and behavior. The chord-layout test covers
triads, octave/input deduplication, adjacent heads, accidental columns, flat
spelling, bass/treble division and notation-range filtering. Stack geometry was
visually inspected with the bundled music font; full desktop testing is pending.

## MIDI playback staff overlay

Open a MIDI song in normal playback and click **Score: Off/On** below the
transport bar. This is independent of Free Play and its exercise modes.

- **View: Scroll** (default): a blue playhead stays at 20% of the time window;
  notes move from right to left past it.
- **View: Pages**: notes stay in place within the current window while the blue
  playhead moves across them. At the boundary the view turns to the next window.
- **Settings → Visible beats** selects 4, 8 or 16 quarter-note beats. These are timing
  windows, not inferred measures or original printed pages.
- **Settings → Staff size** scales notation from 50% to 250%, initially 125%, limited
  to available height and width. The effective fitted percentage is indicated.
- **Settings → Prefer flats** chooses flat rather than sharp spellings for black keys. No key is
  inferred. Each altered note gets an explicit accidental.

Visible pitched MIDI tracks are combined on treble/bass staves, split at middle
C. Hidden tracks, percussion channel 10 and zero-velocity notes are excluded.
Same-onset/same-written-duration tones are stacked, with shared stems and offset
adjacent noteheads. Green means the current note interval at the playhead;
upcoming notes are light, finished notes dim. A paused playhead keeps its current
highlight. Thin rails preserve the exact MIDI durations even when the displayed
note value is only approximate. Notes held across a page/window boundary remain
visible at its left edge, with an explanatory footer. All MIDI pitches can be
shown; extra register room and ledger lines are allocated for the song.

A small **ring on the blue playhead** shows each pitch you play, on its actual
staff position and octave. Green means an exact match with a currently sounding
visible score note; orange means a different pitch; blue means no score note is
active at that instant. The footer shows your pitch and signed distance to the
nearest active target in semitones, e.g. `D4: +2 st from C4`. Octaves count:
playing C5 against C4 reports +12. For chords this compares each input with the
nearest sounding chord tone; it does not judge chord completeness or rhythm.
Feedback is captured at the moment you press, so it does not change when playback
moves to the next note. Released rings remain for 650 ms; sustain and multiple
input channels are respected. Accidentals follow the Prefer flats setting. Notes outside the
song's allocated staff area get a boundary arrow and their true pitch in the
footer, rather than shifting the staff. Up to four pitch comparisons fit in the
footer, with a count for additional notes; all input rings are drawn. File
playback never creates input rings. This works in both Pages and Scroll views.

The overlay reads the player's timestamp after transport actions, subtracts the
existing lead-in, and maps seconds to quarter-note beats using the file's tempo
map. It has no independent animation clock: pause, seek, loop, speed changes,
play-along waiting and tempo changes therefore use the same musical position.
The waterfall's configurable animation offset does not shift the score playhead.
Changing views/windows/size/spelling does not alter playback or send MIDI events.
These view options are saved and shared across song playback scenes.

This is a **timed MIDI practice view**, not a reconstruction of a published
score. Common whole/half/quarter/eighth/sixteenth/32nd values and single dots are
chosen approximately from MIDI durations. It does not infer key/time signatures,
bar structure, rests, ties, tuplets, beaming, voices or hand assignments. Tiny
performance timing differences may produce closely spaced noteheads. Up to 512
notes are rendered per window; dense windows show a prompt to reduce Visible beats in Settings.
The existing piano, transport, waterfall and Free Play practice remain available.

Validation: five standalone score tests cover stationary page positions, moving
and fixed playheads, backward seeking, lead-in, held intervals and duration
symbols, plus exact input pitches, octave errors, nearest chord-tone selection,
stable ties and silent/interval-edge feedback. MIDI tempo tests cover the default tempo, initial/later tempo changes,
negative pre-roll and conversion round trips. Desktop checks:

1. Load a song, enable Score and pause. Note positions/playhead must freeze.
2. Compare Pages and Scroll during playback. Verify moving cursor/still notes
   versus still cursor/moving notes, respectively, including a page boundary.
3. Seek backward/forward, change speed and use looping/play-along waiting.
   Confirm the playhead and note highlights follow the same position as playback.
4. Test a tempo-changing file and long held notes crossing windows. Check
   alignment, carried notes and exact-duration rails.
5. Test chords, flats/sharps, dotted/flagged notes, extremes, hidden tracks,
   percussion, empty score, 4/8/16 beats, zoom and resizing.
6. Pause inside a long C4 note: play C4, D4, B3 and C5; expect green/0,
   orange/+2, orange/−1 and orange/+12 feedback. Check both staves, flats/sharps,
   chords, short taps, sustain/release, duplicate pitches on separate channels,
   out-of-register arrows and silence (blue). Confirm automatic playback creates
   no rings and feedback is captured at attack rather than recolored later.
7. Turn Score off and return to Free Play; confirm the previous note/chord
   practice layouts, hints and optional reference audio behave as before.


## Appearance and saved preferences

The main **Settings** page, Free Play Settings and the in-song Settings drawer share the same
appearance controls. The drawer scrolls and changes the current song's view
without restarting transport. Free Play Settings preserves exercise progress; Escape
closes a drawer before leaving the scene. All presentation preferences save immediately to
the existing settings file and survive restarts. Older settings files receive
safe defaults without replacing device, volume or piano-range selections.

- **Theme:** Midnight (cool blue/teal), Graphite (neutral), Warm (amber), or
  High Contrast. All use dark surfaces with bright notation. High Contrast
  forces an opaque staff background regardless of the transparency preference.
- **Soft gradients:** subtle, stationary diagonal background wash; disable for
  a flat background. There is no decorative background animation.
- **Soft note colors:** a restrained theme palette for the piano and waterfall;
  switch off to use the original/custom configured track colors.
- **Staff size:** 50–250%, shared by Free Play and MIDI playback, initially 125%.
  Fit-to-window still limits effective size. Free Play retains its quick zoom
  buttons; MIDI playback uses Settings to keep the score toolbar compact.
- **Staff background:** 55–100% opacity, initially 88%. Only the rounded panel
  background is translucent; music glyphs and input markers keep their contrast.
- **MIDI staff, Scrolling notation, Visible beats, Prefer flats:** control the
  score's visibility, Scroll/Pages, 4/8/16-beat span and accidental spelling.
  Score and View remain available as quick actions above the staff.
- **Your note marker, Duration guides, Beat grid:** independently hide played
  pitch feedback, exact-duration rails, or the quarter-note timing guides.
- **Play exercise notes:** saved default for optional note/chord reference audio.
  Free Play's Hear task control updates the same preference.

Practice targets, matching rules and note/chord layouts remain unchanged. Colors
are paired with pitch labels and semitone feedback so correctness does not rely
solely on color. Themes do not infer a musical key or alter MIDI/audio timing.

Desktop appearance checks:
1. Change all four themes in Settings, restart and confirm the selection, staff
   size, opacity, score options and exercise-audio preference are restored.
2. Switch themes while a MIDI song plays; the waterfall, controls, staff and
   background should update without restarting or seeking. Check Pages/Scroll.
3. Set opacity to 55% and 100%; only the panel should change, and High Contrast
   should remain opaque. Check duration/grid/marker toggles independently.
4. Scroll the in-song drawer to the last row, including on a small window; no
   settings click should play a piano key underneath the drawer.
5. Check note practice, chord practice and live notes at 50%, 125%, 250% and in a
   resized window. Noteheads, clefs, accidentals and ledger lines stay aligned.
