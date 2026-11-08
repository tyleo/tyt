#!/usr/bin/env python3
"""Measures each run's cost, time, and errors from its transcript and files and
compares two arms prompt by prompt.

Usage: process-metrics.py <round>:<slot>=<arm> ... [--runs]
Each spec puts one slot of a round, 1 or 2, in arm A or B. `live` names the
round in $VOXEL_TRIALS/_runs. A round with its -2 slots on a second arm reads
`r:1=A r:2=B`. A slot-swapped pair adds `s:1=B s:2=A`. Each prompt scores
the mean of arm B's runs minus arm A's for every metric. The scores take a
sign-flip permutation test and a bootstrap interval. `--runs` prints every run's
metrics first.
"""
import json, os, random, re, statistics, sys

root = os.environ.get("VOXEL_TRIALS", os.path.expanduser("~/voxel-trials"))
here = os.path.dirname(os.path.abspath(__file__))
DRAWS = 20_000

METRICS = [
    ("cost", "cost in dollars"),
    ("minutes", "minutes"),
    ("turns", "turns"),
    ("passes", "passes"),
    ("vxlCalls", "vxl commands"),
    ("vxlErrors", "vxl commands that failed"),
    ("buildErrors", "builds that failed"),
    ("toolErrors", "tool calls that failed"),
    ("openChecks", "check lines in the final report"),
    ("staleRender", "model edited after its last render"),
    ("peakContext", "peak context in thousands of tokens"),
    ("compactions", "compactions"),
    ("skillReads", "reads of a skill file"),
]


def run_dirs(round_name, run):
    if round_name == "live":
        return os.path.join(root, "_runs", run), os.path.join(root, run)
    base = os.path.join(root, "rounds", round_name)
    return os.path.join(base, "runs", run), os.path.join(base, "slots", run)


def transcript(record, slot):
    archived = os.path.join(record, "session.jsonl")
    if os.path.exists(archived):
        return archived
    session_id = json.load(open(os.path.join(record, "result.json")))["session_id"]
    project = os.path.expanduser("~/.claude/projects/" + re.sub(r"[^A-Za-z0-9]", "-", slot))
    path = os.path.join(project, f"{session_id}.jsonl")
    if not os.path.exists(path):
        sys.exit(f"no transcript at {path}")
    return path


# A session can read its skill's files with the Read or Grep tool or with a
# shell command, which may reach them by a path relative to the skill.
def skill_read(name, data):
    if name in ("Read", "Grep"):
        return "/.claude/skills/" in data.get("file_path", data.get("path", ""))
    if name == "Bash":
        return bool(re.search(
            r"\b(cat|head|tail|sed|grep|rg|awk|less)\b[^;&|]*(\.claude/skills/|references/)\S*\.md",
            data.get("command", ""),
        ))
    return False


def measure(round_name, run):
    record, slot = run_dirs(round_name, run)
    result = json.load(open(os.path.join(record, "result.json")))
    calls, failed, peak, compactions = {}, set(), 0, 0
    for line in open(transcript(record, slot)):
        entry = json.loads(line)
        if entry.get("isCompactSummary") or entry.get("subtype") == "compact_boundary":
            compactions += 1
        message = entry.get("message") or {}
        usage = message.get("usage") or {}
        peak = max(peak, sum(usage.get(k, 0) for k in ("input_tokens", "cache_read_input_tokens", "cache_creation_input_tokens")))
        content = message.get("content")
        if not isinstance(content, list):
            continue
        for block in content:
            if block.get("type") == "tool_use":
                calls[block["id"]] = (block["name"], block.get("input") or {})
            elif block.get("type") == "tool_result" and block.get("is_error"):
                failed.add(block["tool_use_id"])
    commands = {i: c[1].get("command", "") for i, c in calls.items() if c[0] == "Bash"}
    vxl = {i for i, c in commands.items() if re.search(r"(^|[;&|(\s])vxl\s", c)}
    passes = os.path.join(record, "passes")
    snapshots = sorted(os.listdir(passes)) if os.path.isdir(passes) else []
    open_checks = 0
    if snapshots:
        last = os.path.join(passes, snapshots[-1])
        for name in os.listdir(last):
            if name.endswith("-report.txt"):
                match = re.search(r"^check\s+(\d+) lines?", open(os.path.join(last, name)).read(), re.M)
                open_checks += int(match.group(1)) if match else 0
    models = [n[: -len(".ts")] for n in os.listdir(slot) if n.endswith(".ts")] if os.path.isdir(slot) else []
    stale = any(
        os.path.exists(os.path.join(slot, m + ".png"))
        and os.path.getmtime(os.path.join(slot, m + ".ts")) > os.path.getmtime(os.path.join(slot, m + ".png"))
        for m in models
    )
    return {
        "cost": result["total_cost_usd"],
        "minutes": result["duration_ms"] / 60000,
        "turns": result["num_turns"],
        "passes": len(snapshots),
        "vxlCalls": len(vxl),
        "vxlErrors": len(vxl & failed),
        "buildErrors": sum(1 for i in vxl & failed if "sdf-doc build" in commands[i]),
        "toolErrors": len(failed),
        "openChecks": open_checks,
        "staleRender": int(stale),
        "peakContext": peak / 1000,
        "compactions": compactions,
        "skillReads": sum(1 for name, data in calls.values() if skill_read(name, data)),
    }


def sign_flip(values, rng):
    observed = abs(statistics.mean(values))
    hits = sum(abs(statistics.mean(v * rng.choice((1, -1)) for v in values)) >= observed - 1e-12 for _ in range(DRAWS))
    return hits / DRAWS


args = [a for a in sys.argv[1:] if a != "--runs"]
if not args:
    sys.exit(__doc__)
arms = {"A": [], "B": []}
for spec in args:
    match = re.fullmatch(r"(.+):([12])=([AB])", spec)
    if not match:
        sys.exit(f"{spec} is no <round>:<slot>=<arm> spec")
    arms[match.group(3)].append((match.group(1), match.group(2)))
if not arms["A"] or not arms["B"]:
    sys.exit("each arm takes at least one spec")

prompts = json.load(open(os.path.join(here, "prompts.json")))
runs = {}
for arm, specs in arms.items():
    for round_name, slot_number in specs:
        for p in prompts:
            run = p["dir"] if slot_number == "1" else p["dir"] + "-2"
            runs[(arm, p["dir"], round_name, slot_number)] = measure(round_name, run)

if "--runs" in sys.argv:
    print("arm prompt         round:slot  " + " ".join(f"{k:>11}" for k, _ in METRICS))
    for (arm, d, r, s), m in sorted(runs.items()):
        print(f"{arm}   {d:14} {r[-12:]}:{s}  " + " ".join(f"{m[k]:11.2f}" for k, _ in METRICS))
    print()

rng = random.Random(0)
print(f"arm A {' '.join(f'{r}:{s}' for r, s in arms['A'])}; arm B {' '.join(f'{r}:{s}' for r, s in arms['B'])}")
print(f"{'metric':38} {'A mean':>9} {'B mean':>9} {'B - A':>9} {'95% interval':>20} {'p':>6}")
for key, label in METRICS:
    per_arm = {arm: {p["dir"]: statistics.mean(m[key] for (a, d, _, _), m in runs.items() if a == arm and d == p["dir"]) for p in prompts} for arm in arms}
    diffs = [per_arm["B"][p["dir"]] - per_arm["A"][p["dir"]] for p in prompts]
    boot = sorted(statistics.mean(rng.choice(diffs) for _ in diffs) for _ in range(DRAWS // 10))
    low, high = boot[len(boot) // 40], boot[len(boot) * 39 // 40]
    p = sign_flip(diffs, rng) if any(diffs) else 1.0
    a_mean = statistics.mean(per_arm["A"].values())
    b_mean = statistics.mean(per_arm["B"].values())
    print(f"{label:38} {a_mean:9.2f} {b_mean:9.2f} {statistics.mean(diffs):+9.2f} {f'[{low:+.2f}, {high:+.2f}]':>20} {p:6.3f}")
