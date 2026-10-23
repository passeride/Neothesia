![Neothesia Baner](https://github.com/user-attachments/assets/383438e5-80cd-49d2-af30-85afe5d79c6b)


# Neothesia

Neothesia is a cross-platform MIDI visualizer build in Rust.
It helps people to quickly learn how to play piano.
It takes music notes from a MIDI file as an input and displays them as colorful falling blocks on a virtual piano.

Opensource Synthesia was abandoned in favour of [closed source commercial project](https://www.synthesiagame.com/)  
The goal of this project is to bring Opensource Synthesia back to life, and make it look and work as good (or even better) than commercial Synthesia.

If you have any questions, feel free to join my Discord

[<img alt="Discord" src="https://img.shields.io/discord/273176778946641920?logo=discord&style=for-the-badge&color=%23a051ee">](https://discord.gg/sgeZuVA)

## Screenshots

![image](https://github.com/PolyMeilex/Neothesia/assets/20758186/65483bab-0b74-4fd4-90b1-fdd00508b676)

[![Video](https://github.com/PolyMeilex/Neothesia/assets/20758186/dc564433-aade-4430-b137-5f90000ae9e0)](https://youtu.be/ReE9nVuMCSE)

|![settings](https://github.com/PolyMeilex/Neothesia/assets/20758186/e38642e2-6118-4931-9964-a1df27a36db9)|![track selection](https://github.com/PolyMeilex/Neothesia/assets/20758186/2309d970-0234-45ff-a9f4-105ff08514af)|
|--|--|

[Video](https://youtu.be/ReE9nVuMCSE)

## Download

<a href="https://flathub.org/apps/details/com.github.polymeilex.neothesia"><img width="240" alt="Download on Flathub" src="https://flathub.org/assets/badges/flathub-badge-en.png"/></a>

Arch Linux (**Unofficial AUR** built from source, maintained by @zayn7lie): <https://aur.archlinux.org/packages/neothesia>

All binary releases:
[https://github.com/PolyMeilex/Neothesia/releases](https://github.com/PolyMeilex/Neothesia/releases)

## FAQ

- [FAQ](https://polymeilex.github.io/Neothesia/pages/installation.html)
- [Video encoding](https://polymeilex.github.io/Neothesia/pages/video-encoding.html)

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

Normal MIDI playback also has a **Score** overlay: choose scrolling notation with
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
files. Neothesia remembers the folder and scans it at startup, with optional
subfolders, search, name/newest sorting and pages that fit the window. Click a
piece to play it; use Refresh after changing files. See the
[MIDI library guide](docs/pages/midi-library.md) for details.
