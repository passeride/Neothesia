# ADR-0001: Separate written music, playback and performance targets

- Date: 2026-10-23
- Status: Accepted; implementation pending
- Tracking: ARCH-01, ARCH-04 in TODO.md

## Context

Tangent began with a MIDI playback model. The first MusicXML importer currently
creates a synthetic SMF and attaches written records and original XML to
`MidiFile.score`. `ScoreOverlay` derives notation from timed MIDI pitches instead
of those records. Pitch spelling, rests, staff/voice assignment and meter can
therefore be retained at import yet lost on the path to display. Fixed MIDI tick
resolution also limits otherwise valid input rhythms.

## Decision

Introduce format-neutral domain representations with three distinct purposes:

1. **Written music:** measures, exact musical positions/durations, written pitch,
   rests, voices, staves, clefs, keys and ties, with source references.
2. **Playback plan:** sounding notes and control events on a resolved timeline,
   using a tempo map and explicit repeat/arrangement semantics.
3. **Performance targets:** the immutable selected attacks and rule inputs used
   for a particular Arcade attempt.

Names and crate locations remain implementation choices. Musical position must
support exact rational durations; wall-clock scheduling must use explicit time
units and deterministic conversion. A fixed SMF PPQ must not constrain the
canonical written model.

MIDI import provides playback data and may provide metadata, but does not claim
to contain authoritative written notation. Generated notation is an explicit
fallback. MusicXML import provides written structure and compiles supported
structure into a playback plan. Keep original XML for provenance; do not make
renderers reparse it or infer its structure from MIDI.

Keep references between source notes, written tie segments, sounding notes and
performance targets. They are not necessarily one-to-one. Playback and target
compilers must be independent of window/GPU state and must not write source files.
Mutable user track/visibility choices belong to session/arrangement selection,
not immutable source content.

## Consequences

Staff can use source spelling, rests and staves when available. Repeats and
advanced notation can later be implemented without confusing written position
with playback occurrence. Import errors and support limits remain explicit.

This adds model and mapping work. Introduce adapters around the existing pipeline
in small changes; do not duplicate entire playback engines or redesign all
notation at once. Retain current scoring and MIDI behavior through regression
fixtures. Accurate MusicXML engraving remains separate work.

## Alternatives considered

- **Use MIDI as the universal model:** simple initially, but cannot preserve all
  notation semantics and forces later reconstruction.
- **Render directly from XML:** preserves source detail but couples rendering to
  an import format and leaves timing/scoring mappings unresolved.
- **Replace MIDI support with MusicXML:** excludes existing content and live MIDI
  workflows; MIDI remains a supported input/output boundary.

See the [format roadmap](../plans/formats-and-learning.md) and
[current MusicXML support](../pages/musicxml.md) for delivered behavior.
