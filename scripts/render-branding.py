#!/usr/bin/env python3
"""Regenerate Tangent's SVG wordmark and platform icons.

Requires Python with Pillow/fontTools and Inkscape on PATH. Wordmark lettering
is outlined from the bundled Roboto font, so exported SVGs need no system fonts.
"""
from pathlib import Path
import io
import subprocess

from fontTools.ttLib import TTFont
from fontTools.pens.svgPathPen import SVGPathPen
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
BRAND = ROOT / "assets/branding"

# Outer circle radius 64; line centre is 71 units from its centre.
# The line's 7-unit half-width touches the circle, creating a true tangent.
MARK = '''<path fill="{ink}" fill-rule="evenodd"
  d="M168 132a64 64 0 1 1-128 0a64 64 0 1 1 128 0Z
     M143.84 109a46 30-30 1 0-79.68 46a46 30-30 1 0 79.68-46Z"/>
<path d="M96 240.409 L230 106.409" fill="none" stroke="{accent}"
  stroke-width="14" stroke-linecap="round"/>'''


def svg(body, viewbox, label):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{viewbox}" '
            f'role="img" aria-label="{label}"><title>{label}</title>{body}</svg>\n')


def raster(source, width):
    return subprocess.check_output([
        "inkscape", str(source), "--export-type=png", "--export-filename=-",
        f"--export-width={width}",
    ], stderr=subprocess.DEVNULL)


def main():
    BRAND.mkdir(exist_ok=True)
    (BRAND / "tangent-mark.svg").write_text(svg(
        MARK.format(ink="#f2eee5", accent="#74d7d1"), "0 0 256 256", "Tangent"))
    (BRAND / "tangent-mark-mono.svg").write_text(svg(
        MARK.format(ink="currentColor", accent="currentColor"), "0 0 256 256", "Tangent"))
    icon = svg('<rect width="256" height="256" rx="56" fill="#101b29"/>'
               + MARK.format(ink="#f2eee5", accent="#74d7d1"),
               "0 0 256 256", "Tangent app icon")
    (BRAND / "tangent-icon.svg").write_text(icon)

    font = TTFont(ROOT / "assets/fonts/Roboto-Regular.ttf")
    glyphs, cmap = font.getGlyphSet(), font.getBestCmap()
    scale, x, letters = 0.060, 0, []
    for char in "Tangent":
        glyph = glyphs[cmap[ord(char)]]
        pen = SVGPathPen(glyphs)
        glyph.draw(pen)
        letters.append(f'<path transform="translate({x} 0)" d="{pen.getCommands()}"/>')
        x += glyph.width + 18
    # Same 5.5:1 aspect as the menu's 220 x 40 logical pixel brand area.
    wordmark = svg(
        '<g transform="translate(0 0) scale(.46)">'
        + MARK.format(ink="#f2eee5", accent="#74d7d1") + '</g>'
        + f'<g fill="#f2eee5" transform="translate(165 93) scale({scale} {-scale})">'
        + ''.join(letters) + '</g>', "0 0 660 120", "Tangent")
    (BRAND / "tangent-wordmark.svg").write_text(wordmark)
    (ROOT / "assets/banner.png").write_bytes(raster(BRAND / "tangent-wordmark.svg", 1320))
    png = raster(BRAND / "tangent-icon.svg", 1024)
    image = Image.open(io.BytesIO(png)).convert("RGBA")
    for dest in [ROOT / "assets/tangent-icon.png", ROOT / "docs/public/tangent-icon.png",
                 ROOT / "flatpak/io.github.passeride.Tangent.png"]:
        image.resize((256, 256), Image.Resampling.LANCZOS).save(dest)
    image.save(ROOT / "assets/icon.ico", sizes=[(16, 16), (24, 24), (32, 32),
               (48, 48), (64, 64), (128, 128), (256, 256)])
    image.save(ROOT / ".github/app/Tangent.app/Contents/Resources/icon.icns")
    (ROOT / "docs/public/tangent-icon.svg").write_text(icon)
    print("Generated SVG wordmark, marks, PNG, ICO and ICNS icons.")


if __name__ == "__main__":
    main()
