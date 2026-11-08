#!/bin/sh
# Runs one trial headless in its slot and records the run in _runs/<run>. The
# model and effort come from _runs/round.json. The run puts its arm's vxl first
# on PATH and stops after two hours.
# Usage: run-trial.sh <run>
set -u
here=$(cd "$(dirname "$0")" && pwd)
root=${VOXEL_TRIALS:-$HOME/voxel-trials}
run=$1
out=$root/_runs/$run
prompt=$(jq -r --arg dir "${run%-2}" '.[] | select(.dir == $dir) | .prompt' "$here/prompts.json")
[ -n "$prompt" ] || { echo "no prompt for $run" >&2; exit 2; }
[ -d "$root/$run/.claude" ] || { echo "no slot for $run; run setup.sh" >&2; exit 2; }
model=$(jq -r .model "$root/_runs/round.json")
effort=$(jq -r .effort "$root/_runs/round.json")
case $run in
  *-2) bin=$root/_harness/arm-2/bin ;;
  *) bin=$root/_harness/arm-1/bin ;;
esac
[ -x "$bin/vxl" ] || { echo "no vxl at $bin; run setup.sh" >&2; exit 2; }

mkdir -p "$out"
rm -rf "$out/exit" "$out/finished" "$out/passes" "$out/voxelized" "$out/voxelizes"
printf '%s\n' "$prompt" > "$out/prompt.txt"
date -u +%Y-%m-%dT%H:%M:%SZ > "$out/started"
touch "$out/snapshot-marker" "$out/call-marker"
cd "$root/$run" || exit 2

# A parent Claude Code session's variables would mark the trial as nested.
env -u CLAUDECODE -u CLAUDE_CODE_ENTRYPOINT -u CLAUDE_CODE_MESSAGING_SOCKET \
  -u CLAUDE_CODE_MESSAGING_TOKEN -u CLAUDE_CODE_EXECPATH -u CLAUDE_CODE_SESSION_ID \
  -u CLAUDE_CODE_CHILD_SESSION -u CLAUDE_CODE_SESSION_ATTENDED -u CLAUDE_PID \
  -u CLAUDE_EFFORT -u AI_AGENT \
  PATH="$bin:$PATH" perl -e 'alarm shift; exec @ARGV' 7200 \
  claude -p --model "$model" --effort "$effort" --output-format json \
  < "$out/prompt.txt" > "$out/result.json" 2> "$out/stderr.log"
code=$?
date -u +%Y-%m-%dT%H:%M:%SZ > "$out/finished"
echo "$code" > "$out/exit"
