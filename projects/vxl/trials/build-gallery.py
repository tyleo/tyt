#!/usr/bin/env python3
"""Builds a round's gallery page from its renders, run records, and analyses.

Usage: build-gallery.py [--round <round>] [--findings <dir>]
The page lands in gallery/index.html under $VOXEL_TRIALS for the live round, or
in rounds/<round>/gallery/index.html for an archived one. A findings folder
holds findings.json and the images it references. Each finding lists its
evidence by the ids analysis_items.py prints. Every item has to sit in some
finding.
"""
import base64, json, os, re, shutil, statistics, subprocess, sys

from analysis_items import items

here = os.path.dirname(os.path.abspath(__file__))
root = os.environ.get("VOXEL_TRIALS", os.path.expanduser("~/voxel-trials"))
args = sys.argv[1:]
options = dict(zip(args[::2], args[1::2]))
round_name = options.get("--round")
findings_dir = options.get("--findings")

if round_name:
    base = os.path.join(root, "rounds", round_name)
    runs_dir = os.path.join(base, "runs")
    slot_dir = lambda run: os.path.join(base, "slots", run)
    gallery = os.path.join(base, "gallery")
else:
    runs_dir = os.path.join(root, "_runs")
    slot_dir = lambda run: os.path.join(root, run)
    gallery = os.path.join(root, "gallery")

prompts = json.load(open(os.path.join(here, "prompts.json")))
round_info = json.load(open(os.path.join(runs_dir, "round.json")))
synthesis = json.load(open(os.path.join(runs_dir, "synthesis.json")))


def meters(text):
    match = re.search(r"\d+(\.\d+)?", str(text))
    return float(match.group(0)) if match else None


def size_label(value):
    if value is None:
        return ""
    if value >= 0.1:
        return f"{value:g} m"
    if value >= 0.01:
        return f"{value * 100:g} cm"
    return f"{value * 1000:g} mm"


VIEWS = ["hero", "front", "right", "back", "top"]


def find_sheet(folder):
    """Returns the folder's first review sheet, the PNG named for a report's
    model with or without `-review`, or None."""
    names = sorted(os.listdir(folder))
    models = {n[: -len("-report.txt")] for n in names if n.endswith("-report.txt")}
    return next((n for n in names if n.endswith(".png") and n[:-4].removesuffix("-review") in models), None)


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


def copy_images(run):
    """Crops a run's final review sheet into its views, copies its other PNGs,
    and shrinks each pass's hero view into a data URI. The pass renders ride in
    the page because a published page holds at most 511 files."""
    out = os.path.join(gallery, "runs", run)
    os.makedirs(out)
    sheet = find_sheet(slot_dir(run))
    views = []
    if sheet:
        for index, view in enumerate(VIEWS):
            crop_tile(os.path.join(slot_dir(run), sheet), index, os.path.join(out, view + ".png"))
            views.append(view)
    extras = []
    for name in sorted(os.listdir(slot_dir(run))):
        if name.endswith(".png") and name != sheet:
            shutil.copy(os.path.join(slot_dir(run), name), os.path.join(out, f"extra-{name[:-4]}.png"))
            extras.append(f"extra-{name[:-4]}")
    passes = []
    passes_dir = os.path.join(runs_dir, run, "passes")
    for snapshot in sorted(os.listdir(passes_dir)) if os.path.isdir(passes_dir) else []:
        pass_sheet = find_sheet(os.path.join(passes_dir, snapshot))
        if pass_sheet:
            webp = crop_tile(os.path.join(passes_dir, snapshot, pass_sheet), 0, "-resize", "360x360", "-quality", "82", "webp:-")
            passes.append("data:image/webp;base64," + base64.b64encode(webp).decode())
    return views, extras, passes


shutil.rmtree(os.path.join(gallery, "runs"), ignore_errors=True)
trials = []
for prompt in prompts:
    analysis = json.load(open(os.path.join(runs_dir, prompt["dir"], "analysis.json")))
    runs = []
    for r in analysis["runs"]:
        run = prompt["dir"] if r["run"] == 1 else prompt["dir"] + "-2"
        record = json.load(open(os.path.join(runs_dir, run, "result.json")))
        views, extras, passes = copy_images(run)
        runs.append({
            "run": r["run"], "dir": run, "verdict": r["verdict"], "skillLoaded": r["skillLoaded"],
            "passes": r["passes"], "failedPasses": r["failedPasses"],
            "minutes": round(record["duration_ms"] / 60000, 1),
            "cost": round(record["total_cost_usd"], 2), "turns": record["num_turns"],
            "voxels": r["voxels"], "voxelSize": size_label(meters(r["voxelSize"])),
            "pieces": r["pieces"], "caption": r["caption"], "assessment": r["assessment"],
            "progress": r["progress"], "failures": r["failures"], "flatColors": r["flatColors"],
            "operationsLacked": r["operationsLacked"], "reportDataLacked": r["reportDataLacked"],
            "views": views, "extras": extras, "passImages": passes,
        })
    trials.append({
        "num": prompt["num"], "dir": prompt["dir"], "prompt": prompt["prompt"],
        "stress": prompt.get("stress", ""), "comparison": analysis["comparison"],
        "stressFindings": analysis["stressFindings"], "better": analysis["better"], "runs": runs,
    })

all_runs = [r for t in trials for r in t["runs"]]
stats = {
    "prompts": len(trials),
    "runs": len(all_runs),
    "loaded": sum(r["skillLoaded"] for r in all_runs),
    "good": sum(r["verdict"] == "good" for r in all_runs),
    "fair": sum(r["verdict"] == "fair" for r in all_runs),
    "poor": sum(r["verdict"] in ("poor", "failed") for r in all_runs),
    "medianMinutes": round(statistics.median(r["minutes"] for r in all_runs), 1),
    "cost": round(sum(r["cost"] for r in all_runs), 2),
    "passes": sum(r["passes"] for r in all_runs),
    "failedPasses": sum(r["failedPasses"] for r in all_runs),
}

findings = None
if findings_dir:
    findings = json.load(open(os.path.join(findings_dir, "findings.json")))
    recorded = items(runs_dir, prompts)
    unplaced = set(recorded) - {item_id for finding in findings["findings"] for item_id in finding["evidence"]}
    if unplaced:
        sys.exit(f"no finding holds {', '.join(sorted(unplaced))}")
    for finding in findings["findings"]:
        finding["evidence"] = [recorded[item_id] for item_id in finding["evidence"]]
    shutil.rmtree(os.path.join(gallery, "findings"), ignore_errors=True)
    shutil.copytree(findings_dir, os.path.join(gallery, "findings"), ignore=shutil.ignore_patterns("*.json"))

data = {"round": round_info, "stats": stats, "trials": trials, "findings": findings, "synthesis": synthesis}
template = open(os.path.join(here, "gallery-template.html")).read()
payload = json.dumps(data).replace("</", "<\\/")
with open(os.path.join(gallery, "index.html"), "w") as f:
    f.write(template.replace("/*DATA*/", payload))
print(json.dumps(stats))
