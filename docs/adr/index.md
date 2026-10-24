# Architecture decisions

These ADRs record accepted direction following the architecture review of
23–24 October 2026. **Accepted does not mean implemented.** Delivery and acceptance
criteria are tracked in the root [TODO.md](https://github.com/passeride/Neothesia/blob/feat/live-chord-recognition/TODO.md).

| ADR | Decision | Status | Work |
| --- | --- | --- | --- |
| [0001](0001-music-model.md) | Separate written music, playback and performance targets | Accepted; implementation pending | ARCH-01, ARCH-04 |
| [0002](0002-record-identity.md) | Explicit, versioned record identity encoding | Accepted; implementation pending | ARCH-02 |
| [0003](0003-domain-and-ui-boundaries.md) | Keep performance state and music logic independent of UI and I/O | Accepted; implementation pending | ARCH-03, ARCH-05, ARCH-06 |
| [0004](0004-bevy-clean-room-track.md) | Separate Bevy/web project and documented clean room process | Planning direction accepted; handoff pending | CR-00–07 |

Use a new numbered ADR for a substantial change in direction. Mark an old decision
superseded and link its replacement instead of rewriting its rationale as though
the previous choice never existed. Routine module moves and task details belong
in TODO.md and do not need their own ADR.
