# ADR-0003: Isolate music and performance state from UI and I/O

- Date: 2026-10-23
- Status: Accepted; implementation pending
- Tracking: ARCH-03, ARCH-05, ARCH-06 in TODO.md

## Context

The pure judge in `tangent-core/src/arcade.rs` is already testable. However, the
playing scene also compiles targets, manages the session clock and eligibility,
polls devices, builds/saves records and draws HUD/results/effects. The library
scene combines asynchronous loading, file dialogs, catalog state and rendering.
`tangent-core` also depends on GPU/font infrastructure despite containing pure
music and scoring algorithms.

## Decision

Keep the existing deterministic judge. Extract target compilation and performance
session state behind boundaries that accept explicit data, input and time.
The session returns state/events for presentation and persistence; it must not
require application `Context`, a window, GPU, file dialogs or filesystem access.

The application layer owns adapters for MIDI/audio devices, monotonic timestamps,
file loading and record storage. A source-file freshness check belongs at this
boundary before compilation, not inside the pure compiler. Record-save success
or failure is fed back explicitly so the existing retry behavior is preserved.

Use explicit states/transitions for countdown, playing, pause/re-entry and
results. Record eligibility and persistence status are separate concerns from
animations. Preserve the existing single authoritative timing relationship and
input watermark: song position, calibrated input time and display animation time
must have documented mappings. This decision does not require merging every
normal-playback and Arcade clock into one class.

UI renders state and sends actions. Visual effects consume feedback but do not
change judgement or eligibility. Separate catalog/loading state from library
layout; make ownership of pending asynchronous responses explicit. Share a
format-neutral loading/error policy across entry points.

Establish these as module boundaries first. Extract a small domain crate when it
actually removes GPU/audio dependencies from music and performance tests. Keep
rendering/font/window dependencies in presentation. Do not introduce a generic
plugin framework or dependency-injection container without a concrete need.

## Consequences

Session transitions and compilation can be tested headlessly with controlled
time and input. Rendering changes cannot accidentally change scoring, and new
practice modes can reuse music data without inheriting an entire playing scene.

Adapters and event/result types add some plumbing. Preserve behavior with
characterization tests and incremental moves. Avoid splitting files solely to
meet a line-count target. Existing pause invalidation, calibration, input ordering
and save retries must remain observable and tested.

## Alternatives considered

- **Leave orchestration in the scene:** initially fast, but makes each new mode,
  timing change and persistence failure harder to test independently.
- **Rewrite the whole app around a new engine/framework:** disproportionate to
  the demonstrated problems and risks losing working behavior.
- **Introduce SQLite now:** unrelated to these boundaries. Keep immutable local
  run files until measured storage/query needs justify a separate decision.
