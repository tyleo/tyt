"""Cuts a review sheet into its views."""
import subprocess, sys

VIEWS = ["hero", "front", "right", "back", "top"]


def crop_tile(sheet, index, *convert):
    """Runs magick on one view of a review sheet, which tiles its views three to
    a row with a border between them."""
    width, height = map(int, subprocess.run(
        ["magick", "identify", "-format", "%w %h", sheet], check=True, capture_output=True, text=True,
    ).stdout.split())
    border = 2 * width - 3 * height
    tile = (width - 2 * border) // 3
    if border < 0 or 3 * tile + 2 * border != width:
        sys.exit(f"{sheet} at {width}x{height} is no five-view review sheet")
    x, y = index % 3 * (tile + border), index // 3 * (tile + border)
    return subprocess.run(
        ["magick", sheet, "-crop", f"{tile}x{tile}+{x}+{y}", "+repage", *convert],
        check=True, capture_output=True,
    ).stdout
