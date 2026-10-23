# Working on Tangent

These instructions apply throughout this repository. Tangent is the application
name; the repository URL is still `passeride/Neothesia`. Preserve upstream license
and attribution. Use the original circle-and-tangent logo without a flag.

## Read first

- `TODO.md`: prioritized architecture debt, stable task IDs and completion criteria.
- `docs/adr/index.md`: accepted architecture direction. Accepted ADRs may still be
  unimplemented; check code and TODO status before describing current behavior.
- `docs/pages/musicxml.md` and `docs/pages/arcade.md`: actual support and rules.
- `docs/plans/formats-and-learning.md`: product/format roadmap.

## Architecture rules

- Follow ADR-0001: separate written music, playback and performance targets.
  Preserve source spelling/rests/staves/ties when available; do not add more
  notation reconstruction through synthetic MIDI as the long-term design.
  Keep unsupported musical semantics explicit instead of silently dropping them.
- Follow ADR-0002: persistent identities need explicit, versioned encoding.
  Do not add Debug formatting or randomized collection iteration to hashes.
  Keep scoring-rules changes separate from structural refactoring.
- Follow ADR-0003: keep music logic, target compilation and session transitions
  testable without `Context`, a window, GPU, device or filesystem. Put I/O and UI
  at the application boundary. Preserve calibrated timestamp/watermark behavior.
- Introduce boundaries incrementally. Existing bridge code may remain while a
  TODO task is being delivered; do not claim a task complete until its acceptance
  criteria are met. Avoid unrelated framework/database rewrites.
- MIDI remains first-class. MusicXML/MXL support is a documented subset; a parser
  accepting XML is not proof of correct notation/playback. Clone Hero/Rock Band
  chart formats/importers are out of scope. Difficulty arrangements must retain
  real piano pitches and explicit authoring metadata.
- Backward compatibility with unused prototypes is not a priority. Still version
  identity/rules/schema contracts and never silently mix incompatible records.

## Development and verification

Use meaningful behavior tests for musical timing, import semantics, session
transitions and record identity. Preserve ties as one attack, exact pitch/octave,
repeat-note attacks, calibrated timing and interrupted-run eligibility. Do not
change hit windows or scoring thresholds just to simplify a refactor.

Run checks appropriate to changed code. Current commands include:

```sh
cargo fmt --all -- --check
cargo test -p midi-file --lib --tests --locked
cargo test -p tangent-core --lib --locked
cargo test -p tangent --bin tangent -p nuon --lib --locked
cargo clippy -p midi-file -p tangent --all-targets -- -D warnings
cargo build -p tangent --locked
```

The full CI also lints the workspace and builds platform/recorder targets; see
`.github/workflows/build.yml`. Native Linux builds need development dependencies
for ALSA; recorder targets also need FFmpeg dependencies. Use the platform build
actions as the dependency reference. Report unavailable checks accurately.

For documentation-only changes, check links and run `npm run build` in `docs`
(after `npm ci` if needed); do not require rebuilding the Rust app for prose edits.
Automated tests do not establish physical piano/pedal/latency or Windows GUI
correctness. Distinguish those checks from simulation and compilation.

## Documentation and scope

Keep `TODO.md` and affected support guides current when implementing their tasks.
Use a new numbered ADR for a material architecture decision; supersede old ADRs
explicitly. Routine cleanup does not require an ADR or user approval. Do not
interpret this file as an extra approval gate for work the user authorized.

Mark planned versus delivered behavior clearly. Preserve original input files;
import must not rewrite scores. Maintain bounded XML/archive loading and avoid
external entity/network resolution. Add distributable, attributed fixtures for
new musical semantics and keep them independent of local devices and paths.
