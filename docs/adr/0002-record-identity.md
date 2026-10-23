# ADR-0002: Explicit, versioned encoding for record identity

- Date: 2026-10-23
- Status: Accepted; implementation pending
- Tracking: ARCH-02 in TODO.md

## Context

Arcade category keys currently hash track choices, speed and semantic event data,
including `format!("{:?}", event.message)`. Debug output is not a persistent data
contract: changes to formatting or a dependency can change a key without changing
the played arrangement. Stable metadata IDs alone cannot detect changed music.

## Decision

Retain a content-derived SHA-256 category identity, but define its input as an
explicit, deterministic encoding with an identity-encoding version. Document
field order, event tags, integer widths/byte order, time units, string lengths and
canonical ordering before switching writers.

The identity includes the compiled musical data and effective human/backing/mute
selection and speed needed to distinguish the actual performance, together with
the scoring-rules version. The encoding specification must state which control
and backing events matter. Do not use Debug text, randomized map iteration,
rendering objects or raw import-file bytes as canonical event encoding.

Exclude filename/path, display title, artwork, visibility and other purely
presentational metadata. Do not silently exclude a change that affects targets,
backing behavior or scoring. Equivalent normalization must be explicitly defined
and tested; cross-format equivalence is not assumed merely because titles match.

Version **identity encoding**, **scoring rules** and **record storage schema**
separately. An encoding-only change does not justify altering hit windows or
multipliers. A scoring semantics change requires a rules version change.

Introduce fixed expected-hash vectors and positive/negative equivalence tests.
A key may only change after an intentional contract change, not to make a failing
snapshot pass.

## Consequences

Personal records are reproducible across UI refactors and dependency formatting
changes. This defines a durable boundary without changing the local RON storage
mechanism or introducing a database.

Existing keys use the old contract. Backward compatibility is not a project
priority: migration is optional. Keep legacy records in a separate identity
namespace or explicitly mark them incompatible; never relabel them as new keys
without verifying the corresponding content and category. Document this transition
when the new encoding is implemented.

## Alternatives considered

- **Debug serialization:** convenient but not stable enough for persisted keys.
- **Hash source bytes:** changes on harmless metadata/packaging edits and differs
  across encodings of the same effective performance.
- **Author-supplied IDs only:** useful for content organization, insufficient to
  detect a revised arrangement or changed backing selection.
