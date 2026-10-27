#!/usr/bin/env python3
"""Lists every item a round's analyses recorded, each with the id a finding's
evidence uses.

Usage: analysis_items.py [--round <round>] [<first prompt> <last prompt>]
An id reads <prompt>.<run>.<field><index> for a run's field or
<prompt>.<field><index> for a prompt's. RUN_FIELDS and PROMPT_FIELDS hold the
field letters.
"""
import json, os, sys

RUN_FIELDS = {
    "c": "flatColors", "f": "failures", "o": "operationsLacked", "r": "reportDataLacked",
    "s": "skillMisses", "w": "workarounds",
}
PROMPT_FIELDS = {"k": "comparison", "x": "stressFindings"}


def entries(value):
    return value if isinstance(value, list) else ([value] if str(value or "").strip() else [])


def items(runs_dir, prompts):
    """Returns the analyses' items by id, in prompt order."""
    found = {}
    for prompt in prompts:
        analysis = json.load(open(os.path.join(runs_dir, prompt["dir"], "analysis.json")))
        for run in analysis["runs"]:
            for letter, field in RUN_FIELDS.items():
                for index, text in enumerate(entries(run[field])):
                    found[f"{prompt['num']}.{run['run']}.{letter}{index}"] = {
                        "num": prompt["num"], "run": run["run"], "field": field, "text": text}
        for letter, field in PROMPT_FIELDS.items():
            for index, text in enumerate(entries(analysis[field])):
                found[f"{prompt['num']}.{letter}{index}"] = {
                    "num": prompt["num"], "run": None, "field": field, "text": text}
    return found


if __name__ == "__main__":
    here = os.path.dirname(os.path.abspath(__file__))
    root = os.environ.get("VOXEL_TRIALS", os.path.expanduser("~/voxel-trials"))
    args = sys.argv[1:]
    round_name = None
    if args[:1] == ["--round"]:
        round_name, args = args[1], args[2:]
    runs_dir = os.path.join(root, "rounds", round_name, "runs") if round_name else os.path.join(root, "_runs")
    prompts = json.load(open(os.path.join(here, "prompts.json")))
    first, last = (int(args[0]), int(args[1])) if args else (1, len(prompts))
    for item_id, item in items(runs_dir, prompts[first - 1:last]).items():
        print(f"[{item_id}] ({item['field']}) {item['text']}")
