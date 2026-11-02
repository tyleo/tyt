#!/usr/bin/env python3
"""Renders the final models of a prompt's runs under neutral labels for a judge
that ranks them without knowing which run made which.

Usage: blind-views.py <directory> <order> [--round <round>] [--reference <round>]
<order> lists one source per label A, B, and so on, comma-separated: 1 and 2 for
the round's runs, r1 and r2 for the reference round's. `--round` reads an
archived round in place of the live one. Each model a run reported renders under
the review profile with the installed vxl as <label>-sheet-<n>.png, so every
run shows the same views. The files land in the round's runs/<directory>/judge.
judge-order.json beside that folder maps each label to its source. For a prompt
with shots in prompts.json, each label's model files, reports, and .voxj
documents stay in judge/models/<label> for the shot agents. Prints the files by
label.
"""
import json, os, shutil, string, subprocess, sys

from final_renders import models

here = os.path.dirname(os.path.abspath(__file__))
NAMES = {"1": "run 1", "2": "run 2", "r1": "reference 1", "r2": "reference 2"}

root = os.environ.get("VOXEL_TRIALS", os.path.expanduser("~/voxel-trials"))
args = sys.argv[1:]
directory, order = args[0], args[1].split(",")
options = dict(zip(args[2::2], args[3::2]))
round_name = options.get("--round")
reference = options.get("--reference")

for source in order:
    if source not in NAMES:
        sys.exit(f"{source} is no source; use 1, 2, r1, or r2")
    if source.startswith("r") and not reference:
        sys.exit(f"source {source} takes --reference")


def slot(source):
    run = directory if source.endswith("1") else directory + "-2"
    if source.startswith("r"):
        return os.path.join(root, "rounds", reference, "slots", run)
    if round_name:
        return os.path.join(root, "rounds", round_name, "slots", run)
    return os.path.join(root, run)


prompt = next(p for p in json.load(open(os.path.join(here, "prompts.json"))) if p["dir"] == directory)
runs_dir = os.path.join(root, "rounds", round_name, "runs") if round_name else os.path.join(root, "_runs")
out = os.path.join(runs_dir, directory, "judge")
shutil.rmtree(out, ignore_errors=True)
os.makedirs(out)
labels = {}
for label, source in zip(string.ascii_uppercase, order):
    folder = slot(source)
    labels[label] = NAMES[source]
    files = []
    kept = os.path.join(out, "models", label)
    os.makedirs(kept)
    for index, stem in enumerate(m for m in models(folder) if os.path.exists(os.path.join(folder, m + ".voxj"))):
        name = f"{label}-sheet-{index + 1}"
        for suffix in (".ts", "-report.txt"):
            if os.path.exists(os.path.join(folder, stem + suffix)):
                shutil.copy(os.path.join(folder, stem + suffix), os.path.join(kept, f"model-{index + 1}{suffix}"))
        model = os.path.join(kept, f"model-{index + 1}.voxj")
        shutil.copy(os.path.join(folder, stem + ".voxj"), model)
        subprocess.run(
            ["vxl", "object", "render", model, "--profile", "review", "--to", "png", "--file-stem", name],
            check=True, capture_output=True,
        )
        shutil.move(os.path.join(kept, name + ".png"), os.path.join(out, name + ".png"))
        files.append(name + ".png")
    if not prompt.get("shots"):
        shutil.rmtree(kept)
    print(f"{label}: {', '.join(os.path.join(out, f) for f in files) or 'no model'}")

if not prompt.get("shots"):
    os.rmdir(os.path.join(out, "models"))
with open(os.path.join(runs_dir, directory, "judge-order.json"), "w") as f:
    json.dump(labels, f)
