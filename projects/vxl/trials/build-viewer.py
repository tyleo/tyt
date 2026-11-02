#!/usr/bin/env python3
"""Builds a page for ranking a judged round's models by eye, one large image at
a time.

Usage: build-viewer.py [--round <round>]
Each prompt's judge folder, which blind-views.py and the shot agents fill,
becomes one image strip per label holding the sheet's five views and the shots.
A view a model lacks shows as a blank tile. The page lands in viewer/ under
$VOXEL_TRIALS for the live round, or under rounds/<round> for an archived one.
It saves the places to its artifact's db.
"""
import json, os, shutil, subprocess, sys, tempfile

from review_sheet import VIEWS, crop_tile

SIZE = 900
MAX_STRIP = 16383
FONT = "/System/Library/Fonts/Supplemental/Arial.ttf"

here = os.path.dirname(os.path.abspath(__file__))
root = os.environ.get("VOXEL_TRIALS", os.path.expanduser("~/voxel-trials"))
options = dict(zip(sys.argv[1::2], sys.argv[2::2]))
round_name = options.get("--round")
base = os.path.join(root, "rounds", round_name) if round_name else root
runs_dir = os.path.join(base, "runs") if round_name else os.path.join(root, "_runs")
viewer = os.path.join(base, "viewer")


def blank(path, text):
    subprocess.run(
        ["magick", "-size", f"{SIZE}x{SIZE}", "xc:#1b1b1b", "-fill", "#8a8a8a", "-gravity", "center",
         "-font", FONT, "-pointsize", "48", "-annotate", "+0+0", text, path],
        check=True, capture_output=True,
    )


prompts = json.load(open(os.path.join(here, "prompts.json")))
shutil.rmtree(viewer, ignore_errors=True)
os.makedirs(os.path.join(viewer, "img"))
data = []
with tempfile.TemporaryDirectory() as scratch:
    for prompt in prompts:
        folder = os.path.join(runs_dir, prompt["dir"], "judge")
        order_path = os.path.join(runs_dir, prompt["dir"], "judge-order.json")
        if not os.path.exists(order_path):
            sys.exit(f"{prompt['dir']} has no judge-order.json; judge the round first")
        labels = sorted(json.load(open(order_path)))
        sheets = max(
            [n for label in labels for n in range(1, 100) if os.path.exists(os.path.join(folder, f"{label}-sheet-{n}.png"))],
            default=1,
        )
        shots = [
            (k, shot) for k, shot in enumerate(prompt.get("shots", []), 1)
            if any(os.path.exists(os.path.join(folder, f"{label}-shot-{k}.png")) for label in labels)
        ]
        views = [view if sheets == 1 else f"{view} {n}" for n in range(1, sheets + 1) for view in VIEWS]
        views += [f"shot {k}: {shot}" for k, shot in shots]
        if len(views) * SIZE > MAX_STRIP:
            sys.exit(f"{prompt['dir']} has {len(views)} views, too many for one strip")
        for label in labels:
            tiles = []
            for n in range(1, sheets + 1):
                sheet = os.path.join(folder, f"{label}-sheet-{n}.png")
                for index in range(len(VIEWS)):
                    tile = os.path.join(scratch, f"{len(tiles)}.png")
                    if os.path.exists(sheet):
                        crop_tile(sheet, index, "-resize", f"{SIZE}x{SIZE}", tile)
                    else:
                        blank(tile, "no model" if n == 1 else f"no model {n}")
                    tiles.append(tile)
            for k, _ in shots:
                shot = os.path.join(folder, f"{label}-shot-{k}.png")
                tile = os.path.join(scratch, f"{len(tiles)}.png")
                if os.path.exists(shot):
                    subprocess.run(["magick", shot, "-resize", f"{SIZE}x{SIZE}", tile], check=True, capture_output=True)
                else:
                    blank(tile, "not modeled")
                tiles.append(tile)
            subprocess.run(
                ["magick", *tiles, "+append", "-quality", "80", os.path.join(viewer, "img", f"{prompt['dir']}-{label}.webp")],
                check=True, capture_output=True,
            )
        data.append({"dir": prompt["dir"], "prompt": prompt["prompt"], "labels": labels, "views": views})
        print(f"{prompt['dir']}: {len(labels)} models, {len(views)} views")

page = open(os.path.join(here, "viewer.html")).read()
with open(os.path.join(viewer, "index.html"), "w") as f:
    f.write(page.replace("/*DATA*/[]", json.dumps(data)))
print(os.path.join(viewer, "index.html"))
