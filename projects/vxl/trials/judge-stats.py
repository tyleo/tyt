#!/usr/bin/env python3
"""Unblinds a round's judge rankings and compares two arms prompt by prompt.

Usage: judge-stats.py [--round <round>] [--arm-a <sources>] [--arm-b <sources>]
Sources are comma-separated names such as "run 1" or "reference 2". Arm B
defaults to the -2 slots, run 2 and reference 2, and arm A to the other sources.
Each prompt scores the mean rank of arm A minus that of arm B over every
judging. A positive score favors arm B. The prompts' scores take a sign-flip
permutation test and a bootstrap interval. Stability counts the source pairs
whose order held across every judging of a prompt.
"""
import glob, json, os, random, statistics, sys

root = os.environ.get("VOXEL_TRIALS", os.path.expanduser("~/voxel-trials"))
here = os.path.dirname(os.path.abspath(__file__))
options = dict(zip(sys.argv[1::2], sys.argv[2::2]))
runs_dir = os.path.join(root, "rounds", options["--round"], "runs") if "--round" in options else os.path.join(root, "_runs")
DRAWS = 100_000


def sources_of(option):
    return [s.strip() for s in options[option].split(",")] if option in options else None


def sign(x):
    return (x > 0) - (x < 0)


scores, stable, pairs = {}, 0, 0
for prompt in json.load(open(os.path.join(here, "prompts.json"))):
    folder = os.path.join(runs_dir, prompt["dir"])
    order = json.load(open(os.path.join(folder, "judge-order.json")))
    verdicts = [json.load(open(p)) for p in sorted(glob.glob(os.path.join(folder, "judge", "verdict-*.json")))]
    if not verdicts:
        sys.exit(f"{folder} holds no verdicts")
    ranks = [{order[label]: rank for label, rank in v["ranks"].items()} for v in verdicts]
    names = sorted(order.values())
    arm_b = sources_of("--arm-b") or [n for n in names if n.endswith("2")]
    arm_a = sources_of("--arm-a") or [n for n in names if n not in arm_b]
    for name in arm_a + arm_b:
        if name not in names:
            sys.exit(f"{prompt['dir']} has no source {name}")
    mean = {n: statistics.mean(r[n] for r in ranks) for n in names}
    scores[prompt["dir"]] = statistics.mean(mean[n] for n in arm_a) - statistics.mean(mean[n] for n in arm_b)
    for i, x in enumerate(names):
        for y in names[i + 1:]:
            pairs += 1
            stable += len({sign(r[x] - r[y]) for r in ranks}) == 1
    print(f"{prompt['dir']:14} {scores[prompt['dir']]:+.2f}  " + "  ".join(f"{n} {mean[n]:.2f}" for n in names))

values = list(scores.values())
n = len(values)
observed = statistics.mean(values)
rng = random.Random(0)
extreme = sum(abs(statistics.mean(v * rng.choice((1, -1)) for v in values)) >= abs(observed) - 1e-12 for _ in range(DRAWS))
boot = sorted(statistics.mean(rng.choice(values) for _ in range(n)) for _ in range(DRAWS // 10))
low, high = boot[len(boot) // 40], boot[len(boot) * 39 // 40]
print(f"\narm A {', '.join(arm_a)}; arm B {', '.join(arm_b)}")
print(f"arm B better on {sum(v > 0 for v in values)}, worse on {sum(v < 0 for v in values)}, even on {sum(v == 0 for v in values)} of {n} prompts")
print(f"mean score {observed:+.3f}, 95% interval [{low:+.3f}, {high:+.3f}], permutation p = {extreme / DRAWS:.3f}")
print(f"stability: {stable} of {pairs} source pairs held their order across every judging ({stable / pairs:.0%})")
