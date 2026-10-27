#!/usr/bin/env python3
"""Prints a trial session's transcript as plain text, with images and long tool
output cut down.

Usage: summarize-transcript.py [--round <round>] <run> [limit]
For an archived round, the script reads the run's copy of the transcript. For a
live run, the script reads the transcript in Claude Code's project folder for
the run's slot.
"""
import glob, json, os, re, sys

root = os.environ.get("VOXEL_TRIALS", os.path.expanduser("~/voxel-trials"))
args = sys.argv[1:]
round_name = None
if args[:1] == ["--round"]:
    round_name, args = args[1], args[2:]
run = args[0]
limit = int(args[1]) if len(args) > 1 else 4000
if round_name:
    out = os.path.join(root, "rounds", round_name, "runs", run)
else:
    out = os.path.join(root, "_runs", run)
slot = os.path.join(root, run)
project = os.path.expanduser("~/.claude/projects/" + re.sub(r"[^A-Za-z0-9]", "-", slot))

# Attachments that only repeat session setup.
NOISE = {
    "command_permissions", "credential_org", "date", "deferred_tools_record",
    "hook_success", "prompt_snapshot", "remote_session_change",
    "session_context", "total_tokens_reminder",
}


def transcript_path():
    archived = os.path.join(out, "session.jsonl")
    if os.path.exists(archived):
        return archived
    try:
        with open(os.path.join(out, "result.json")) as f:
            session_id = json.load(f).get("session_id")
    except (OSError, ValueError):
        session_id = None
    if session_id and os.path.exists(os.path.join(project, f"{session_id}.jsonl")):
        return os.path.join(project, f"{session_id}.jsonl")
    newest = sorted(glob.glob(os.path.join(project, "*.jsonl")), key=os.path.getmtime)[-1:]
    if not newest:
        sys.exit(f"no transcript for {run}")
    return newest[0]


def cut(text, n=limit):
    text = text if isinstance(text, str) else json.dumps(text)
    return text if len(text) <= n else text[:n] + f"\n... [{len(text) - n} more chars]"


def tool_input(name, data):
    if name == "Write":
        return f"{data.get('file_path')}\n{cut(data.get('content', ''), 2500)}"
    if name == "Edit":
        return (f"{data.get('file_path')}\n--- old\n{cut(data.get('old_string', ''), 1200)}"
                f"\n+++ new\n{cut(data.get('new_string', ''), 1200)}")
    if name == "Bash":
        return data.get("command", "")
    return cut(json.dumps(data), 1500)


def entries():
    for line in open(path):
        try:
            yield json.loads(line)
        except ValueError:
            pass


def is_pass(block):
    """A pass is a call the snapshot hook saw write a voxj or a voxelize command
    that failed. A round from before the hook kept that list counts each
    voxelize command."""
    command = block.get("input", {}).get("command", "")
    if voxelizes is None:
        return "sdf-doc voxelize" in command
    return block.get("id") in voxelizes or ("sdf-doc voxelize" in command and block.get("id") in failed)


path = transcript_path()
log = os.path.join(out, "voxelizes")
voxelizes = set(open(log).read().split()) if os.path.exists(log) else None
failed = {
    block.get("tool_use_id")
    for entry in entries()
    if isinstance((entry.get("message") or {}).get("content"), list)
    for block in entry["message"]["content"]
    if block.get("type") == "tool_result" and block.get("is_error")
}
print(f"# transcript {path}")
passes = 0
for entry in entries():
    kind = entry.get("type")
    if kind == "attachment":
        attachment = entry.get("attachment") or {}
        if attachment.get("type") not in NOISE:
            print(f"\n[attachment] {cut(json.dumps(attachment), 300)}")
        continue
    if entry.get("isMeta"):
        print(f"\n[{kind} meta] {cut(json.dumps(entry.get('message', {}).get('content')), 200)}")
        continue
    if kind not in ("user", "assistant"):
        continue
    content = entry.get("message", {}).get("content")
    if isinstance(content, str):
        print(f"\n## {kind.upper()}\n{cut(content)}")
        continue
    for block in content or []:
        btype = block.get("type")
        if btype == "text":
            print(f"\n## {kind.upper()} TEXT\n{cut(block['text'])}")
        elif btype == "tool_use":
            name = block.get("name")
            data = block.get("input", {})
            if name == "Bash" and is_pass(block):
                passes += 1
                print(f"\n## PASS {passes}")
            print(f"\n## TOOL {name}\n{tool_input(name, data)}")
        elif btype == "tool_result":
            parts = block.get("content")
            if isinstance(parts, list):
                parts = "\n".join(
                    part["text"] if part.get("type") == "text" else f"[{part.get('type')}]"
                    for part in parts
                )
            flag = " ERROR" if block.get("is_error") else ""
            print(f"\n## RESULT{flag}\n{cut(parts or '')}")
print(f"\n# {passes} passes")
