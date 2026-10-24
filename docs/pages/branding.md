# Tangent identity

Tangent is the application name across piano practice, playback and the planned
Arcade mode. The source repository URL still uses `passeride/Neothesia`.

The symbol combines a circular whole-note head, a slanted open counter and a
diagonal tangent. The line's inner edge meets the outer circle at one point.
Ivory and turquoise on midnight navy connect the mark to the Midnight Resonance
Arcade concept. A single-colour SVG is available for printing and small UI marks.

![Tangent app icon](/tangent-icon.svg)

## Source assets

- `assets/branding/tangent-mark.svg`: transparent two-colour symbol.
- `assets/branding/tangent-mark-mono.svg`: symbol using `currentColor`.
- `assets/branding/tangent-wordmark.svg`: symbol plus outlined Tangent lettering.
- `assets/branding/tangent-icon.svg`: square app icon with a dark rounded surface.

The SVGs contain paths rather than external images or fonts. The wordmark uses
outlines derived from the existing bundled Roboto font. Keep the mark's aspect
ratio and leave clear space around it; do not crop the tangent endpoints.

`python scripts/render-branding.py` regenerates the wordmark and the PNG, Windows
ICO and macOS ICNS exports. It requires Inkscape, Pillow and fontTools. The menu,
window icon, executable resource, app bundle, Linux launcher and docs all use
exports from the same geometry. Raster exports are committed so normal Rust
builds do not need the artwork tools.

## Application names

The GUI executable is `tangent` (`tangent.exe` on Windows), the recorder is
`tangent-cli`, the macOS bundle is `Tangent.app`, and the Linux application ID is
`io.github.passeride.Tangent`. Workspace packages/directories use `tangent`,
`tangent-core`, `tangent-image`, `tangent-cli` and `tangent-ai`.

Linux preferences use `$XDG_CONFIG_HOME/tangent/settings.ron` or
`~/.config/tangent/settings.ron`. Windows preferences stay beside the executable;
macOS keeps the bundled resource behavior. This rename deliberately has no
legacy-name aliases or configuration migration. Existing upstream distribution
packages and old screenshots are identified as upstream rather than advertised
as Tangent builds.

The Flatpak manifest is a local source-build template using this checkout.
Building it requires network access to Cargo dependencies; a sandboxed/offline
release needs a newly generated vendor source archive before distribution.
The new application ID is not a claim of publication on Flathub.
