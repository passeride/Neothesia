# ADR-0004: Plan a separate Bevy/web implementation with a clean room process

- Date: 2026-10-24
- Status: Accepted planning direction; process handoff, implementation and final dependency selection pending
- Scope: A new project, not a relicensing or engine migration of this fork
- Tracking: CR-00 through CR-07 in the [plan](../plans/bevy-clean-room.md)

## Context

The user requests a clean room implementation without Neothesia, using Bevy and
targeting browsers. Existing fork code, local wrappers, assets and design material
cannot be assumed suitable for a source-independent handoff. This assistant has
already worked on that code; the current conversation is not an unexposed
implementation environment. Changing engine or creating a fresh repository alone
does not establish independent authorship.

## Decision

Plan a new repository with independent history and a documented specification /
implementation separation. Review an implementation-neutral requirements package
and its rights/provenance before handing it to the implementation side. Record
actual source exposure rather than claiming that a new AI context erases it.
This ADR and its linked planning documents are management material, not an already
reviewed clean room handoff. No new implementation code is authorized by this ADR
alone; implementation scope follows the user's instructions.

Use Bevy as presentation/application framework and keep a platform-independent
music/practice domain. Target web from the first vertical slice. Evaluate Bevy
0.20 with compatible published packages; do not move fork crates into the new
project or downgrade solely for an old MIDI plugin. Independently sourced
third-party libraries remain candidates under their own licenses.

Treat browser MIDI, audio scheduling, file access and persistence as explicit
platform capabilities. Prototype timestamped MIDI, Web Audio/AudioWorklet,
imported-file persistence and both rendering profiles before broad feature work.
Do not equate successful WASM compilation with working browser audio or devices.

Keep the existing repository, history, LICENSE and attribution unchanged. ADRs
0001–0003 and ARCH tasks still describe the existing fork's work; they are not
silently superseded or completed by this new plan. Reusing their text or designs
in an implementation handoff also requires review.

## Consequences

We gain a deliberate path toward a web-first standalone product and evidence of
its origin. We also accept process overhead and avoid assuming automatic reuse
of recently written fork code, fixtures or branding. A permissive license is a
possible new-project goal only after rights and dependency review; it is not a
claim about the existing GPL fork or a legal guarantee of clean room status.

A small custom MIDI/audio adapter is the initial recommendation. OSMD is a
candidate for web notation; native parity and mapping to performance occurrences
remain separate work. Final package versions follow a real prototype, not this
ADR's existence.

## Alternatives

- Port the fork into Bevy: reasonable for continued GPL development, but does not
  meet the requested source-independence goal merely through an engine change.
- Share the fork's local music crates: expedient, but imports its code/provenance.
- Build native first and add WASM later: leaves the largest audio/MIDI/storage
  constraints unresolved until too late.

See the [library assessment](../plans/bevy-libraries.md) and the plan's primary
sources. Legal/provenance assessment and measured technical feasibility are
separate questions, and both must be represented honestly.
