![Tangent](assets/branding/tangent-preview.png)


# Tangent

Tangent is a cross-platform MIDI visualizer build in Rust.
It helps people to quickly learn how to play piano.
It takes music notes from a MIDI file as an input and displays them as colorful falling blocks on a virtual piano.

Tangent builds on [Neothesia](https://github.com/PolyMeilex/Neothesia), with piano practice, live chord recognition and a persistent MIDI library.

## Upstream screenshots (before Tangent branding)

![image](https://github.com/PolyMeilex/Neothesia/assets/20758186/65483bab-0b74-4fd4-90b1-fdd00508b676)

[![Video](https://github.com/PolyMeilex/Neothesia/assets/20758186/dc564433-aade-4430-b137-5f90000ae9e0)](https://youtu.be/ReE9nVuMCSE)

|![settings](https://github.com/PolyMeilex/Neothesia/assets/20758186/e38642e2-6118-4931-9964-a1df27a36db9)|![track selection](https://github.com/PolyMeilex/Neothesia/assets/20758186/2309d970-0234-45ff-a9f4-105ff08514af)|
|--|--|

[Video](https://youtu.be/ReE9nVuMCSE)

## Download

Download Tangent builds from this repository's [GitHub Actions](https://github.com/passeride/Neothesia/actions) artifacts, or [Releases](https://github.com/passeride/Neothesia/releases) when published. Windows builds are named `tangent.exe`; macOS bundles are `Tangent.app`.

The upstream Flathub/AUR packages distribute Neothesia, not this Tangent fork.

## Documentation

- [New Bevy/web implementation plan (Norwegian)](docs/plans/bevy-clean-room.md) and [library assessment](docs/plans/bevy-libraries.md) — planning only; this repository remains the Neothesia-derived Tangent fork.

- [Technical debt and refactoring tasks](TODO.md)
- [Architecture decisions](docs/adr/index.md)

- [MusicXML piano import](docs/pages/musicxml.md) — `.musicxml`, `.xml` and `.mxl` playback, with a documented subset and generated Staff.

- [Formats and learning roadmap (Norwegian)](docs/plans/formats-and-learning.md)
- [Development blog: Tangent og den første Arcade-modusen](docs/blog/2026-10-23-tangent-arcade.md)
- [Installation](docs/pages/installation.md)
- [Video encoding](docs/pages/video-encoding.md)
- [Brand assets](docs/pages/branding.md)

## Thanks to

- [WGPU](https://wgpu.rs/)
- [Linthesia](https://github.com/linthesia/linthesia)
- [Synthesia](https://github.com/johndpope/pianogame)

## Live chords in Free Play

Enable **Chords** in Free Play to identify played MIDI chords, inversions, and
alternative interpretations. See [chord recognition](docs/pages/chords.md) for
the API, supported vocabulary, sustain behavior, and verification steps.

Enable **Training** beside the Chords switch to practice 20 common chords with
chord names, note names, keyboard markers, and automatic progression after a
correct answer.

Enable **Notes** for live treble/bass staff notation, or **Note practice** to learn
reading individual notes. Choose treble, bass or both, optional sharps, and hints.
Single-note practice continues without a score or ending and requires the correct octave.

Normal MIDI playback also has a **Staff** overlay: choose scrolling notation with
a fixed playhead or stationary pages with a moving playhead. Staff size, beat
window and accidental spelling are adjustable. A small ring at the playhead shows
your played pitches: green for a match, orange for a mismatch, with semitone
distance to the nearest active target in the footer.


**Appearance** in Settings offers Midnight, Graphite, Warm and High Contrast
themes, soft static gradients, and restrained piano/waterfall colors. Staff
size and background opacity are shared by practice and MIDI notation. The
in-song Settings drawer also controls notation movement, visible beats,
accidentals, played-note feedback, duration guides and beat grid. Preferences
are saved automatically; exercise audio remains optional.


## MIDI library

Choose a folder on the home screen to keep a persistent gallery of your MIDI
files. Tangent remembers the folder and scans it at startup, with optional
subfolders, search, name/newest sorting and pages that fit the window. Click a
piece to play it; use Refresh after changing files. See the
[MIDI library guide](docs/pages/midi-library.md) for details.


## Arcade performance

Choose **Arcade** on a MIDI card, select Play Along target tracks, and play to a
fixed timeline with Perfect/Great/Good judgements, combo and ×1–×4 multipliers.
A perfect complete run earns 1,000,000 points. Results show accuracy, timing,
misses/extras and personal-best progress; eligible records survive restart and
appear on gallery covers with their track/speed category.

Manual input calibration and effects can be adjusted before play. Pausing keeps
a practice result rather than replacing an uninterrupted record. The original
MIDI arrangement is supported now; authored difficulty packages and custom cover
art remain planned. See the [Arcade guide](docs/pages/arcade.md).
