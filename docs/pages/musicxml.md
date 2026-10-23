# MusicXML piano import

Tangent can open `.musicxml`, `.xml` (with a MusicXML root) and compressed `.mxl`
files from the library, **Select File**, or the command line. This is an initial,
bounded **piano playback import**, not complete MusicXML notation rendering.
Files stay unchanged; no intermediate MIDI is written to disk.

Opening an imported score from the library/file picker shows track selection
before playback. Tracks are named by source part ID, staff and voice. Choose
Auto Play, Play Along or Mute there, or switch to Arcade. Staff numbers are not
an automatic claim about which hand to use.

## Supported musical structure

- `score-partwise` documents, pitched notes and rests, simultaneous chords.
- Multiple parts, up to two staves per part, and multiple voices through explicit
  `backup` / `forward` durations. Each pitched part/staff/voice has its own track;
  at most 15 are supported, avoiding the MIDI percussion channel.
- Divisions and duration values, including exactly representable tuplets. The
  playback resolution is 15,120 ticks per quarter; unsupported fractions fail
  rather than being rounded onto a different rhythm.
- Simple time signatures and changes at measure boundaries. Ordinary measures
  pad to their meter; `implicit="yes"` measures use their actual extent, including
  pickups. Parts must agree on measure boundaries. Missing meter defaults to 4/4.
- Integer alterations from double flat to double sharp, MIDI pitches 0–127.
  Notes sound at the pitch explicitly encoded in `<pitch>`; key signatures do
  not apply a second accidental to those pitches.
- Contiguous sound ties (`<tie>` start/stop) across measures become one attack
  and one sustained note. Their written segments are retained separately.
- Explicit `<sound tempo="…">`, including direction offsets and tempo changes.
  Default tempo is 120 BPM; supported explicit tempos are 4–1000 BPM. Conflicting
  tempos at the same position fail. A metronome marking without a sound tempo
  currently requires re-export with explicit playback tempo.

The importer creates an in-memory playback plan through Tangent's existing MIDI
pipeline. Accepted imports can therefore use playback, generated Staff and the
existing attack-scoring Arcade rules. Each attack uses piano sound and velocity
80. Expressive dynamics, articulations and page layout are not reproduced.

## Notation retained, rendering still limited

`MidiFile.score` retains the original XML, title, source track labels and written
note/rest records: part, measure, staff, voice, musical position, duration,
diatonic spelling, written type, dots and tie flags. Key signatures, clefs,
beams, fingering and other source notation remain available in the original XML.

The current Staff renderer still reads playback pitches and generates its own
notation. It does **not** yet render the source's exact spelling, voices, clefs,
rests, beams or page layout. The track-selection notice makes this distinction
explicit. Raw XML retention is not a promise that every visual element is used.

## Explicit limits

`score-timewise` and opus documents are not imported. Repeats, endings, playback
jumps, grace/cue notes, unpitched notes, transposition, octave shifts, ornaments,
rolled-chord instructions, glissandi, fermatas, pedal instructions and repeat
measure/beat shorthand are rejected. So are unsupported additive/mid-measure
meters, non-contiguous/unclosed ties, overlapping unisons within one voice, and
chords that cross staff/voice or exceed their preceding note's duration.

These cases need deliberate playback semantics before they can be scored. Export
a linear, simplified piano score when necessary. Removing a repeat marker without
expanding its music is not an equivalent arrangement.

Some engraving-only elements (such as slurs, stem direction and spacing) are
retained but ignored by playback. This is not full schema validation. Plain XML
with another root gets an import error when opened; library scanning lists file
extensions without parsing all documents, so unrelated `.xml` files can appear.

## Compressed files and limits

MXL loads its first declared rootfile through `META-INF/container.xml`; the
importer never extracts files to disk. Missing roots, duplicate names, absolute
or traversing paths are rejected. Limits: 32 MiB source file, 1,024 archive
entries, 32 MiB declared total uncompressed content, 16 MiB per read XML document,
500,000 XML nodes and 100,000 written notes/rests. XML is currently UTF-8 only.

External DOCTYPE identifiers in exporter headers are inert: no DTD or network
resource is fetched. Entity declarations and internal DTD subsets are rejected.

## Example and verification

The original [Tangent Piano Study](https://github.com/passeride/Neothesia/blob/feat/live-chord-recognition/midi-file/tests/fixtures/piano-study.musicxml)
is a CC0 test score with a pickup, two staves, 3/4 meter, a chord, a tie, rests and
a tempo change. Open it via Select File to try the importer.

Automated tests cover these timings and written properties, repeated pitches,
exact triplets, plain/compressed file opening, extension handling, rejection of
unsupported playback structures, broken ties, conflicting tempo, malformed XML,
archive paths and oversized input. App-level checks cover compilation into the
existing Arcade target model. Hardware MIDI and Windows GUI behavior still need
hands-on testing.

Next: a typed notation model and rendering from the retained score, broader
exporter fixtures, then deliberate repeat/tuplet/engraving support. See the
[format roadmap](../plans/formats-and-learning.md).

## Primary references

- [MusicXML 4.0: notation basics](https://www.w3.org/2021/06/musicxml40/tutorial/notation-basics/)
- [MusicXML 4.0: compressed MXL files](https://www.w3.org/2021/06/musicxml40/tutorial/compressed-mxl-files/)
