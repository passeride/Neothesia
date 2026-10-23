# Arcade mode: gameplay, scoring and presentation plan

Status: **initial playable implementation delivered; remaining features are a roadmap**.
The [Arcade guide](../pages/arcade.md) describes current behavior and limits.
The original design was prepared 2026-10-23 against
`4c76887553c7b5fc3cc4f9872fb25e047a506569`; the application is now Tangent.

Implemented: deterministic attack scoring, callback timestamps, fixed-time MIDI
play, track/speed selection, manual calibration, countdown, bounded hit effects,
combo/tier HUD, pause with practice eligibility, results, persistent personal
records and procedural gallery covers with labelled best scores.

Remaining: authored difficulty packages, real cover-art loading, guided/per-device
calibration, richer stage effects and Windows/physical-piano validation. The
initial record store uses immutable, committed RON run files rather than SQLite.
The sections below preserve the intended full design; they are not all current UI.

## Product direction

Add **Arcade** alongside normal playback: a complete, timed piano performance
with score, combo, multiplier, personal records and an expressive result screen.
Keep the existing endless practice modes. Call the existing notation display
**Staff**, since “Score” must not mean both notation and game points.

The experience is: gallery → song/difficulty/hand selection → short countdown →
performance → results → retry or gallery. Default to finishing the song even
after mistakes. Recovery should feel possible; a compulsory failure meter would
interrupt learning. A separate survival challenge can follow later.

MVP means a playable vertical slice with reliable judgement, records, cover cards
and essential hit effects. The later polish stages complete the full presentation.
Do not release only a decorative HUD around the current play-along statistics.

## Formats: research and decision

User constraint: **Clone Hero is inspiration for game feel only. Do not implement
its data format or a guitar-chart importer. All supported note data must preserve
actual piano pitches and arrangements.** The comparison below records research,
not a proposed compatibility roadmap.

| Format | Useful properties | Limitation / decision |
| --- | --- | --- |
| Ordinary MIDI | Timed pitches, velocities, tempo changes and multiple tracks; existing parser and synthesizer | Tracks are not inherently difficulty levels. Keep working unchanged as a standalone “Original” arrangement. |
| Clone Hero song folder | `notes.mid` or `notes.chart`, `song.ini`, audio; explicit game conventions and metadata | Useful packaging pattern. Five-fret MIDI notes encode lanes and difficulty, not ordinary piano pitches. Do not interpret these as piano arrangements. |
| Rock Band Pro Keys MIDI | Named Easy/Medium/Hard/Expert tracks; closer to an actual keyboard chart | The documented playable pitch range is MIDI 48–72, with game markers and range shifts. Research reference only; do not adopt or implement an importer. |
| MusicXML / compressed `.mxl` | Written notation interchange; a better source for spelling, voices and staff notation than reconstructing everything from MIDI | Does not by itself establish Tangent's game rules or difficulty selection. Later notation import, not a prerequisite for Arcade. |
| MusicXML opus collection | Groups multiple MusicXML scores; can represent a collection containing separate piano arrangements | A usable standard collection mechanism, but the Easy/Medium/Hard relationship would still need an explicit convention. Candidate for later piano-score interchange. |
| Synthesia `.synthesia` sidecar | Piano-oriented metadata alongside unchanged MIDI: title, composer, arranger, difficulty, hands, finger hints and song groups | A relevant existing piano precedent. Its documented difficulty is a per-song rating; the guide does not establish a standard multi-difficulty arrangement selector for one song. Use as inspiration, not a dependency. |
| Tangent song folder with JSON manifest | Existing MIDI files plus explicit arrangements, title, artist, cover and stable IDs | Recommended first implementation. An open, versioned sidecar convention, not a replacement note encoding. |

Clone Hero's `diff_keys` / `diff_guitar` metadata are displayed intensity ratings;
they are not the Easy/Medium/Hard note charts themselves. The core distinction is
**song → arrangement/difficulty → playable part**, not “one MIDI track per level”
unless the author explicitly declares that mapping.

Primary references consulted:

- [Clone Hero: Adding Custom Songs](https://wiki.clonehero.net/books/clone-hero-manual/page/adding-custom-songs)
- [Clone Hero: song.ini Guide](https://wiki.clonehero.net/books/guides-and-tutorials/page/songini-guide)
- [YARG repository](https://github.com/YARC-Official/YARG), which links the following chart reference
- [Guitar Game Chart Formats: five-fret MIDI](https://thenathannator.github.io/GuitarGame_ChartFormats/Chart-File-Formats/mid-format/Tracks/5-Fret-Guitar/)
- [Guitar Game Chart Formats: Pro Keys](https://thenathannator.github.io/GuitarGame_ChartFormats/Chart-File-Formats/mid-format/Tracks/Pro-Keys/)
- [MusicXML introduction](https://www.w3.org/2021/06/musicxml40/tutorial/introduction/)
- [MusicXML compressed MXL container](https://www.w3.org/2021/06/musicxml40/tutorial/compressed-mxl-files/)
- [MusicXML opus collection reference](https://www.w3.org/2021/06/musicxml40/musicxml-reference/elements/opus-reference/)
- [Synthesia: Creating Great Content](https://synthesiagame.com/support/guide/contentCreators)

The native-format choice and scoring below are our design conclusions, not rules
claimed by those sources. This research did not identify a documented, widely
interoperable piano-game format with a ready-made Easy/Medium/Hard arrangement
contract. That is a limited finding, not a claim that no such format exists.
MusicXML collections and Synthesia's MIDI-plus-metadata approach both support
the proposed direction. A generic ZIP distribution container and optional audio
can follow once the native piano song-folder convention is stable.

### Proposed song manifest

Use `song.tangent.json` in a song folder. Paths are relative to that folder.
Example structure: manifest, `cover.png`, `easy.mid`, `medium.mid`, `hard.mid`.
Multiple difficulties may reference different track selections in the same MIDI.

```json
{
  "schema_version": 1,
  "song_id": "example-author:evening-study",
  "title": "Evening Study",
  "artist": "Example Composer",
  "charter": "Example Arranger",
  "cover": "cover.png",
  "arrangements": [
    {
      "id": "easy",
      "difficulty": "easy",
      "revision": 1,
      "midi": "easy.mid",
      "parts": [
        { "id": "right", "label": "Right hand", "tracks": [1] }
      ]
    },
    {
      "id": "hard",
      "difficulty": "hard",
      "revision": 1,
      "midi": "hard.mid",
      "parts": [
        { "id": "right", "label": "Right hand", "tracks": [1] },
        { "id": "left", "label": "Left hand", "tracks": [2] }
      ]
    }
  ]
}
```

Track indices refer to zero-based source SMF tracks, including a conductor track
if present; convert explicitly to internal track IDs. The UI must show track
names and pitch ranges so authors can verify their selection. Non-selected
tracks can be backing or muted, as chosen before starting. “Both hands” is the
union of declared parts. An optional intensity rating is separate from difficulty.

Validate schema, unique IDs, file presence and track bounds. Resolve paths inside
the song folder, including symlinks; reject escapes and absolute paths. Report a
bad arrangement without hiding valid sibling arrangements. Ignore unknown
optional fields, reject unsupported major schema versions. Never modify source
MIDIs. A recognized manifest owns its referenced MIDIs in the gallery, avoiding
duplicate cards; unrelated files in that folder remain standalone entries.

For a standalone MIDI: infer title from filename, use a generated cover, allow
explicit human-track selection and call the arrangement **Original**. Do not
guess Easy/Hard from file size, tempo or track count. Later, deterministic
reductions may create clearly marked **Auto Easy** arrangements with a versioned
generation recipe. Slowing down Hard is still Hard at a different speed.

## Scoring: rewarding timing and sustained consistency

### What counts as a target

Compile only the selected human parts to immutable targets: ID, MIDI pitch,
onset in integer microseconds, duration, source track and onset-group ID. Convert
through the complete tempo map. Ignore percussion and non-note markers. Collapse
duplicate targets with exactly the same pitch and onset across selected tracks;
one physical key cannot satisfy two separate unisons. Keep source provenance.

An onset group is the unique set of pitches starting on the same source tick.
It can be a single note or a chord. Do not merge arbitrary nearby attacks into
chords: that can destroy trills, arpeggios and intentional rolled chords. Authored
roll/group metadata can be added later under a new chart/rules version.

Exact pitch and octave are required. A held note or sustain pedal cannot hit a
later repeated note: that requires a fresh physical NoteOn. Velocity is expressive
output data, not a score multiplier. Quiet playing should earn the same points.
Out-of-keyboard-range targets must be disclosed before starting; offer a different
part or practice mode, never silently omit them from the recorded chart.

### Timing profile v1 (initial tuning)

Measure `error = calibrated_input_time - target_time`; negative is early.
Windows are real milliseconds, including when playing at reduced speed.

| Judgement | Absolute error | Quality points per note before multiplier |
| --- | --- | --- |
| Perfect | ≤ 45 ms | 100 |
| Great | > 45 ms and ≤ 90 ms | 80 |
| Good | > 90 ms and ≤ 150 ms | 50 |
| Miss | No matching attack by +150 ms | 0 |

Only NoteOn with nonzero velocity is an attack; velocity zero is a release.
Match each attack to at most one unmatched target of the same pitch inside its
window, minimizing absolute error; ties choose the earlier target. Commit the
match once, never reassign it after another attack arrives. Overlapping windows
for repeated notes follow this same deterministic rule. Pedal/controllers and
NoteOff never count as extra notes. Preserve legitimate rapid NoteOn retriggers;
do not apply an arbitrary musical debounce. Device duplication must be diagnosed
or resolved at input routing rather than forgiving repeated attacks globally.

Input that matches nothing is an **Extra**, including an early attack outside
the window. Do not report that as a pitch error if it is the right pitch at the
wrong time. Attacks count from first target −150 ms through last target +150 ms;
countdown warmup is outside the run and cannot pre-hit a target. Empty target
charts cannot start a scored run.

### Combo and points

Combo counts **completed onset groups**, so a six-note chord advances the streak
once, while earning six notes' worth of timing points. All required chord pitches
must hit to continue the combo; a partial chord still receives partial credit.

| Consecutive completed groups before the current group | Multiplier |
| --- | --- |
| 0–9 | ×1 |
| 10–24 | ×2 |
| 25–49 | ×3 |
| 50+ | ×4 |

Resolve a group at its fixed onset +150 ms deadline, in chart order. Use the
multiplier determined by the streak **before** that successful group, then
increment the streak. This makes group 11 the first ×2 group, independent of
the order in which a MIDI device transmits a chord's notes.

A group containing a Miss resets the streak before awarding its remaining hits
at ×1. An Extra subtracts 100 raw points and resets the streak at its input time.
Adjudication processes Extras and group deadlines in timestamp order, Extras
first at equal timestamps. A later successfully resolved group can begin a new
streak. Never reset the streak again just to replay an old Extra. Hit feedback
appears immediately, while combo/points settle at group deadlines; avoid showing
a speculative multiplier that can be taken away retroactively.

```text
raw = sum(group_multiplier × sum(note_quality_points)) − 100 × extra_attacks
maximum = raw score of a simulated all-Perfect, no-Extra run on this exact chart
score = floor(1_000_000 × clamp(raw, 0, maximum) / maximum)
```

Use signed integer accumulation and wide checked arithmetic. Clamp only the
display/final normalized value, not the running raw accumulator: otherwise
mashing while at zero would make penalties disappear. Show a small explicit
penalty cue on Extra. A perfect full run always earns 1,000,000. This is a
chart-relative achievement measure, not a claim that Easy and Expert require
equal skill; records are never compared across arrangements or modifiers.

Examples:

- A C/E/G chord at ×2, played Perfect/Great/Good, earns
  `2 × (100 + 80 + 50) = 460` raw points and advances combo once.
- With E missing, that chord earns `100 + 0 + 50 = 150` at ×1 and resets combo.
- Holding C through the next written C does not hit it. A fresh attack is needed.
- Playing an extra F costs 100 and breaks the current streak, even if C/E/G all
  hit. Playing every key cannot produce a clean run or a maximum score.
- For 12 single-note groups, the perfect maximum is
  `10 × 100 + 2 × 200 = 1,400`. All Great gives 1,120 raw / 800,000 normalized.
  Missing group 6 and playing the others Perfect gives 1,100 raw / 785,714.

Classic combo weighting makes a missed note cost both its own points and some
future multiplier opportunity. That is deliberate Arcade tension. The separate
accuracy measure below explains musical progress without hiding it behind combo.
Tune streak thresholds after short beginner pieces as well as long dense pieces;
do not tune exclusively on expert material.

### Results that teach something

Always show score, personal-best delta, hit coverage, Perfect/Great/Good/Miss/Extra
counts, longest group streak and early/late timing distribution.

`accuracy = sum(note_quality_points) / (100 × (target_count + extra_count))`.
Coverage is `matched_target_count / target_count`. This prevents “100% accuracy”
from mashing every key. Do not silently call coverage accuracy.

Use 1–5 stars at 200k / 400k / 600k / 800k / 950k as initial score milestones.
Below 200k show completion and progress without a failure message. Award **Full
Combo** only for no Miss and no Extra; **Perfect Run** additionally requires all
Perfect judgements. Optional per-section summaries can identify where to practice.
Best accuracy and best score may come from different runs; display them honestly.

### Why hold scoring is deferred

Ordinary MIDI release lengths do not reliably describe intended finger release,
legato or pedal technique. Initially judge attacks only; long notes still render
tails and physical/pedal feedback. Show “Attack scoring” before play and in results.
Later introduce an explicitly authored sustain profile with time-based rewards,
documented CC64 semantics and bounded duration weighting. Give it a distinct rules
version and records. Do not reward holding the pedal forever or farming NoteOns.
Likewise, no random bonuses or activatable double-score power in v1: improving
timing should be the most direct route to a better record.

## Timing and integration: the first engineering dependency

Current `PlayingScene::update` stops playback while required keys are missing;
`MidiPlayer::PlayAlong` also uses a 500 ms recent-key cache and frame-time
`Instant::now()` statistics. These are appropriate to assisted play-along, not
this scoring specification. Keep that path and implement a separate Arcade engine.

Timestamp MIDI at the callback boundary, before the UI event queue. Extend
`TangentEvent::MidiInput` with a monotonic arrival timestamp (or a mapped
backend timestamp when available). Define a run clock anchored to monotonic time,
with exact speed, lead-in and pause segments. Derive target positions, judgement
deadlines and MIDI scheduling from that mapping, not accumulated render deltas.
Map input through the segment active at arrival, not the segment active when its
queued event is finally processed. Drain timestamped input before advancing the
judgement watermark; a stalled UI must not miss notes it has already received.

Calibration needs a guided repeated-tap test plus manual adjustment, separately
recording input/output timing correction and visual display offset. Give the
offset a documented sign and test synthetic positive/negative delays. Save per
device/output combination. Visual scroll speed is cosmetic; song speed changes
the target clock and record category. Calibration correction alone is not a
difficulty category. Changing it mid-run invalidates the run.

Arcade must bypass auto-wait, preserve Auto backing tracks and mute file NoteOns
on human target tracks while forwarding user input to the chosen output. Never
feed synthesized backing or task audition into the scoring input. Check existing
controller forwarding for shared MIDI channels so backing CC64/AllNotesOff cannot
accidentally interfere with human input/output voices. Gate held keys across
countdown, pause and device reconnection. On device loss pause safely and label
the attempt interrupted; do not silently turn it into misses.

Use an explicit state machine: selection, countdown, running, paused, finishing,
results, aborted. Finish only after the last judgement window and a short audio
tail, not merely MIDI end-of-track. Seeking, looping, speed/part changes and
auto-wait switch to a clearly labelled practice attempt; they cannot replace a
complete-run record. Paused/interrupted attempts have separate practice results
in v1. Escape opens pause; exiting a run requires a deliberate menu action.

## Visual direction and effects inventory

The application is now **Tangent**. Use its circular note-and-tangent symbol and
ivory/turquoise wordmark (see [branding](../pages/branding.md)). Arcade retains
the **Midnight Resonance** concept: midnight navy, ivory, turquoise and warm amber,
with local resonance rings at the strike line. The main logo stays consistent
across playback, practice and Arcade.

Use the existing Midnight/Graphite/Warm/High Contrast themes: restrained stage
lighting, translucent surfaces, crisp musical content and brighter local feedback.
Keep the piano's full pitch identity rather than replacing it with five lanes.
Avoid making the whole screen compete with the next notes.

| Element / trigger | Appearance and motion | Readability / implementation boundary |
| --- | --- | --- |
| Song selection | Square cover, title/artist, difficulty chips, hand selection, best score and stars | Explicit Original/Auto labels and speed; preview target range and scoring profile before Start |
| Intro / countdown | Cover/title briefly fades into stage; 3–2–1 on the beat | Stable strike line visible throughout; no scoring during warmup |
| Stage | Subtle dark gradient, soft vignette and hand accents | Contrast checked per theme; no moving texture behind dense note clusters |
| Waterfall | Clear onset edges, softly shaded bodies and quiet sustain tails | Human targets strongest, backing subdued; retain black/white key alignment and octave guides |
| Strike line | Thin bright rail just above keys with a restrained glow | This is the exact visible judgement location; visual calibration must align it |
| Perfect | Small expanding ring, brief white-to-accent bloom, 6–10 short sparks, compact judgement | Approx. 180–300 ms; local to the key; no repeated labels per chord pitch |
| Great / Good | Smaller accent ring; Good can add an Early/Late cue | Use text/shape as well as colour; do not make playable notes look like errors |
| Miss / Extra | Short amber/red outline on missed/wrong key; missed onset fades past rail | Approx. 250 ms; no default camera shake or full-screen red flash |
| Held key / pedal | Thin sustained key glow and subdued tail illumination | Feedback only under attack scoring, never imply hidden hold points |
| Combo HUD | Clear streak, ×1–×4 badge, segmented progress to next tier | Top peripheral area; 120–180 ms scale pulse only on tier change |
| Streak milestone | Brief accent sweep at 10 / 25 / 50 groups and restrained particle fan | ≤ 500 ms; intensity capped; no visual obstruction or extra scoring rule |
| Score HUD | Stable-width digits, accuracy, progress and personal-best target | Smooth number interpolation is presentation only; no layout jitter |
| Staff overlay | Optional compact translucent staff with playhead and played-note marker | Reserve space so HUD, staff and approaching notes do not overlap |
| Beat / section | Quiet beat-line pulse from tempo map; section name fades in | Never derive rhythm from rendering FPS or random effect timing |
| Pause / recovery | Dimmed stage, clear Resume/Restart/Exit, short re-entry countdown | Clear held input; paused run marked practice for records |
| Finish / new best | Let final notes breathe; restrained cover reveal, score count-up and one PB celebration | Result values immediately available; skip animation supported |
| Results | Cover, stars, judgement counts, timing plot, best delta, Retry / Gallery | Make one useful improvement suggestion; no wall of diagnostics |
| Gallery record | Dark translucent badge over lower cover with best points, stars and FC icon | Badge explicitly matches selected difficulty/hand/speed; never mix categories |

At narrow window sizes use one compact HUD row, collapse secondary counters and
offer a smaller or hidden staff. Keep strike line and upcoming notes unobstructed.
Expose effects intensity Off/Subtle/Full, reduced motion, particle toggle, high
contrast, judgement text size, staff size/opacity and optional UI sounds in Settings.
Reduced motion replaces animated bursts with static brief highlights; do not
remove necessary hit/error information. UI sounds must not mask the piano.

Start with pooled GPU-batched rings/quads and a bounded particle system (initial
budget: 512 live particles, at most 64 spawned per frame; coalesce chord effects).
Reuse the quad/text renderers. Approximate glow with alpha layers first; add a
separate bloom pass only if measured quality warrants its GPU cost. Effects may
be dropped under load, input and judgement may not. Measure 1080p/60 performance
on Windows integrated graphics, dense chords, rapid repeats and large staff size.

## Cover art and durable personal records

The current gallery renders text cards, not actual covers. Cover loading and the
record overlay are separate required work. Manifest art takes precedence; loose
`piece.mid` may use `piece.cover.png` / `.jpg`, otherwise a deterministic themed
fallback. Do not give every unrelated MIDI in a folder the same `album.jpg`.
Decode thumbnails asynchronously with dimension/file limits and a bounded texture
cache; visible page first. Bad artwork gets a fallback, not a broken song.

Store records in the application's writable data directory, never beside songs
or in general visual settings. Use a small SQLite database with transactional
run insertion and best-record updates, schema migrations and a bundled SQLite
dependency for predictable Windows packaging. Preserve a failed-to-save result
on screen with an explicit Retry action; never announce “record saved” on failure.

Record key: canonical chart content hash + selected parts/backing configuration +
difficulty/arrangement ID + exact speed + transposition + timing/scoring rules
version + assistance flags. Hash canonical target pitches/onsets/durations and
tempo interpretation with a versioned compiler; include generated-reduction recipe
when applicable. Source path, title and cover are excluded, so moving/renaming a
piece or changing artwork preserves records. A changed arrangement keeps old
history but starts a fresh current-chart record. A song ID alone is insufficient.

Store run ID/time, key, raw/max/normalized score, counts, accuracy numerator and
denominator, max streak, completion/eligibility, calibration profile and optional
timestamped replay. Best-score ties use accuracy, fewer Extras, then earliest
achievement; keep best accuracy separately. Local records are not an anti-cheat
or online competitive service. No accounts or network leaderboard needed.

On a gallery card show the best completed eligible run for the currently selected
arrangement/part at 100% speed by default. Show difficulty and a clear `—` if
unplayed; show 80% or other speed explicitly when selected. Refresh the badge
immediately after results and persist through restart. A “Last played” summary
may link to another category, but must not masquerade as the displayed record.

## Implementation boundaries

Proposed new modules (names may adjust to the existing architecture):

- `tangent-core/src/arcade/chart.rs`: target compiler, groups, canonical identity.
- `tangent-core/src/arcade/judge.rs`: timestamped matching and deterministic deadlines.
- `tangent-core/src/arcade/scoring.rs`: pure rules, perfect maximum, result summary.
- `tangent-core/src/song_package.rs`: manifest schema and validation; extend library
  catalog without parsing every MIDI during startup.
- App `arcade/session.rs`: run clock, lifecycle, timestamp routing and playback policy.
- App `arcade/presentation.rs`: HUD, effect pool and renderer integration.
- App `records.rs`: SQLite boundary and record queries off the render thread.
- Existing `menu_scene/library.rs`: cover cache, arrangement selector and record badges.
- Existing input manager / `main.rs`: preserve callback timestamps to the session.

Judgement emits typed events (Hit, Miss, Extra, GroupResolved, TierChanged,
Finished). Presentation consumes them; particles, frame rate, colour choices and
text animations never write back into judgement. Persist a replay of original
input timestamps plus settings to reproduce disputes and regression fixtures.

## Delivery sequence and acceptance gates

1. **Clock, chart and scoring core.** Implement pure matching, rule versioning,
   timestamp mapping and replay fixtures before UI polish. Gate: replay produces
   identical outcomes at 30/60/144 FPS and after artificial render stalls; all
   examples above and boundary cases pass.
2. **Playable Arcade slice on existing MIDI.** Add mode selection, target tracks,
   countdown, fixed-time play, basic score/combo HUD, immediate key feedback and
   results. Gate: physical Windows keyboard/MIDI playtest, backing isolation,
   calibration, pause/device-loss handling and final-window completion. Keep
   practice/play-along regression checks.
3. **Packages, difficulty, cover cards and records.** Add the manifest loader,
   per-arrangement choices, thumbnail cache and transactional records. Gate:
   complete Easy and Hard on one song, restart, move the folder and still see
   the correct distinct records over the correct cover. A chart edit must not
   inherit an incompatible record.
4. **Full visual treatment.** Implement the effect inventory, polished countdown,
   tier transitions, themed stage, responsive HUD, result animation and PB reveal.
   Gate: screenshots/video in every theme and narrow/large windows, reduced-motion
   checks and measured particle/render budget on Windows integrated graphics.
5. **Content and tuning.** Author a small progression of piano arrangements and
   compare novice/intermediate playtests. Tune windows/stars/combo once, bump the
   rules version when needed, then build a Windows test artifact with change notes.
6. **Later extensions.** MusicXML notation/collection import, explicit sustain/pedal charts,
   generated reductions, piano arrangement editing, audio stems and optional special
   phrase mechanics. Each new scoring mechanic requires versioned comparable records.

### Required verification matrix

| Area | Cases / invariant |
| --- | --- |
| Timing | ±45/90/150 ms inclusive boundaries, tempo changes, nonzero first note, lead-in, reduced speed, calibration sign |
| Matching | Chords in every input order, duplicate unisons, repeated pitch with overlapping windows, trills, rolled attacks, velocity-zero release |
| Fairness | Partial chord, rapid wrong notes, all-key mash, held repeated target, pedal-only input, legitimate retriggers, no backing/autoplay credit |
| Arithmetic | Perfect = 1,000,000, all-Miss = 0, Extras never increase score, no negative display/overflow, example scores, empty chart rejected |
| Lifecycle | Late final hit, long tail, pause/re-entry, seek/restart, device loss, queued input during UI stall, no record for incomplete/assisted run |
| Records | Distinct speed/hand/difficulty/rules, rename/move, changed chart, tie policy, read-only song folder, database write failure and migration |
| Catalog | Manifest ownership without duplicate cards, missing arrangement, invalid paths, Unicode, broken art, large library without blocking UI |
| Presentation | All themes, high contrast, reduced motion, muted UI sounds, narrow window, dense chords, effects disabled with identical score |

The largest uncertainties are physical input/output latency, the quality of
arbitrary MIDI arrangements, and sustained-note semantics. None blocks Arcade,
but each must be addressed explicitly rather than hidden by generous scoring or
decorative effects. A few curated piano charts will make the initial experience
substantially better than assigning difficulty names to arbitrary MIDI files.
