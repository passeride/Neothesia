# Arcade: scored piano performance

Arcade is a separate, fixed-time mode for MIDI songs. It keeps moving when you
miss, so you can recover and finish the piece. Normal playback and endless
note/chord practice remain available. Tangent keeps the original circle-and-
tangent logo without a flag.

## Start a run

1. Choose **Arcade** on a song card, or load a piece with **Select File** and
   choose **Arcade** in the bottom bar.
2. Mark the tracks you will play as **Play Along**. Choose **Auto Play** for
   backing and **Mute** for parts you do not want to hear. Track numbers and
   pitch ranges help identify the parts; tracks are not automatically hands or
   difficulty levels.
3. Choose 25–200% speed, optional input calibration, and **Effects: On/Off**.
4. Click **Start Arcade**, then play after the three-second countdown. Match
   the exact pitch and octave. Target tracks stay visible and are not auto-played.

The first implementation uses the MIDI's **Original** arrangement. Easy/Medium/
Hard packages, MusicXML import and authored sustain scoring are later work in
the [Arcade roadmap](../plans/arcade-mode.md). There is no Clone Hero importer.

Empty target selections and targets outside the configured keyboard range show
an error before play. Adjust the keyboard range in Settings or choose another
part; notes are never silently dropped to improve a record. Piano and percussion
must be in separate tracks for Arcade; mixed tracks show a clear error. If the source file
changed since loading, reload it from the gallery.

## Points and combo

| Judgement | Timing error, early or late | Base points |
| --- | --- | --- |
| Perfect | 0–45 ms | 100 |
| Great | More than 45, up to 90 ms | 80 |
| Good | More than 90, up to 150 ms | 50 |
| Miss | No matching attack inside the window | 0 |

Every physical attack can hit one target of the same pitch. For rapid repeated
notes, the nearest unmatched target inside the window wins; equal distances
choose the earlier note. Exact duplicate unisons are one target. Sustain and
NoteOff are not attacks, and holding a key cannot answer a later repeated note.
Velocity does not affect points.

A complete simultaneous chord advances combo once. Each pitch still earns its
own timing points. A partial chord earns its remaining note points at ×1 and
resets combo. Chords use exact onset timestamps, without merging nearby rolls
or trills. One unmatched attack during the scoring interval is an **Extra**:
it costs 100 raw points and resets combo. The interval runs from 150 ms before
the first target to 150 ms after the last target.

Multipliers are ×1 initially, ×2 after 10 complete groups, ×3 after 25, and ×4
after 50. The new multiplier applies to the next successful group. Groups
resolve at the end of their 150 ms late window; local hit feedback is immediate.

The displayed score is `1,000,000 × earned / perfect maximum`, rounded down.
The maximum simulates a perfect run on that exact chart, including multiplier
progression. A perfect complete run earns **1,000,000**, regardless of piece
length. This is progress within a specific arrangement, not a global difficulty
ranking. Missing early can also cost future multiplier opportunities.

Accuracy measures timing quality, with Extras in the denominator. During play
it uses hits/misses judged so far; results use the complete chart. Coverage and
judgement counts are shown separately. **Full Combo** requires no Miss or Extra;
**Perfect Run** also requires every attack to be Perfect. Star milestones are
200k, 400k, 600k, 800k and 950k, initially subject to playtesting.

Only attacks are scored. Sustain tails and pedal remain useful visual/audio
feedback, but release duration and pedal technique do not carry hidden penalties.

## Calibration, effects and interruptions

Positive input offset subtracts a known input delay: an attack arriving 20 ms
late with `+20 ms` calibration is judged at the intended time. Negative offsets
work in the opposite direction. This manual setting is saved, with a ±250 ms
range in 5 ms steps. A guided calibration tool and per-device profiles are still
planned. Waterfall animation offset is a separate visual adjustment configured
before play; timing is measured against the song clock.

Effects add bounded rings, small Perfect sparks, multiplier pulses and a result
count-up. **Off** disables those animations while retaining judgement text and
all scoring information. Midnight, Graphite, Warm and High Contrast themes are
shared with the rest of the app. **Staff** toggles the existing notation display;
the old “Score” toggle has been relabelled to avoid confusion with points.

**Space**, **Escape**, or **Pause** opens pause; use Resume or Retry from start.
Resume includes a two-second re-entry countdown. Focus loss and disappearance of
a connected MIDI input also pause the run. A paused/interrupted run can be
finished for feedback, but is labelled **practice result** and cannot replace
an uninterrupted record. The pause/results panels offer **Gallery** to exit.
On results, **R** also retries from the beginning.

Seeking, loops, mid-run speed changes and playback shortcuts are unavailable in
Arcade. Those remain part of normal playback. Input queue overflow or stale
timestamps invalidate record eligibility rather than quietly discarding attacks.

## Results and records

Results show points, stars, accuracy, Perfect/Great/Good/Miss/Extra counts,
longest combo, coverage, mean early/late error and a timing histogram. The screen
stays open until Retry or Gallery. An eligible completed run saves automatically;
a write failure stays visible with **Retry save**.

Records are separated by a SHA-256 identity of the semantic MIDI event data,
selected human/backing/mute tracks, exact speed and scoring rules version. Path,
filename, artwork and visibility are not part of the identity. File changes
create a new record category when relevant chart/backing data changes.

The gallery shows the best known record over a procedural note-bar cover, with
**Original**, speed and track numbers next to it. Different categories remain
distinct; the card is a labelled summary, not a comparison of Easy and Hard.
Small windows use a compact cover and preserve the category in the subtitle.
Custom cover-art loading is not included in this slice.

Records live in `arcade-runs` beside the settings file, not in the song folder.
Each completed run is a separate RON file, written to a unique temporary file,
flushed, then renamed into place. Existing runs are never overwritten. The app
rebuilds its best-record index at startup and skips unreadable/incompatible files
with a gallery warning. This initial store replaces the roadmap's proposed
SQLite dependency; indexed SQLite storage remains an option for large histories.

Gallery lookup uses cached path/size/modification-time associations to avoid
reading every MIDI at startup. After moving/renaming a file, completing a run
re-associates that location while retaining its old best score by chart identity.
The record comparison itself always uses the compiled chart hash. These are local
records, not an online leaderboard or anti-cheat system.

Backing output is remapped away from the human piano channel in Arcade, so its
pedal/panic messages do not control the player's voice. Channel 10 retains drums;
the other 14 backing channels are reused for songs with more backing tracks.
Physical MIDI device/output behavior still needs testing on real hardware.

## Verification

Automated checks cover exact timing boundaries, chord permutations and unisons,
partial credit, repeated pitches, early/late ties, mashing, multiplier examples,
all-Miss/perfect scores, stale input, empty/invalid charts, replay equivalence at
30/60/144 FPS and after frame stalls, calibration signs, pause clock mapping,
record commits and reload, chart identity, backing-channel isolation, queue
overflow, and quick mouse clicks between rendered frames.

Native Linux checks under software rendering covered empty-selection errors,
countdown, an all-Miss completion, a three-note Perfect Run at 1,000,000 points,
result persistence, gallery record badges, minimum-size gallery layout and pause.
Windows GUI, physical Yamaha input, pedal behavior and output latency require
hands-on testing; automated timing tests do not measure real device latency.
