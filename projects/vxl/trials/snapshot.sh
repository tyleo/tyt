#!/bin/sh
# PostToolUse hook: after a Bash call that writes PNGs, copies the slot's new
# PNGs, model files, and reports into _runs/<run>/passes/<n>. PNGs with no voxj
# written since the last snapshot join that pass. The files a call writes decide
# because a session can voxelize and render from a script. _runs/<run>/voxelizes
# lists the tool use of each call that wrote a voxj.
input=$(cat)
slot=${CLAUDE_PROJECT_DIR:-$PWD}
out=$(dirname "$slot")/_runs/$(basename "$slot")
[ -f "$out/snapshot-marker" ] || exit 0
newer() { (cd "$slot" && find . -path ./.claude -prune -o "$@" -print); }
if [ -n "$(newer -name '*.voxj*' -newer "$out/call-marker")" ]; then
  printf '%s\n' "$input" | jq -r '.tool_use_id' >> "$out/voxelizes"
  touch "$out/voxelized"
fi
touch "$out/call-marker"
pngs=$(newer -name '*.png' -newer "$out/snapshot-marker")
[ -n "$pngs" ] || exit 0
n=$(ls -d "$out"/passes/[0-9]* 2> /dev/null | wc -l | tr -d ' ')
[ -f "$out/voxelized" ] || [ "$n" = 0 ] && n=$((n + 1))
pass=$out/passes/$(printf '%02d' "$n")
mkdir -p "$pass"
(cd "$slot" && printf '%s\n' "$pngs" | while read -r f; do cp "$f" "$pass/"; done)
(cd "$slot" && find . -path ./.claude -prune -o \( -name '*.ts' -o -name '*-report.txt' \) -print | while read -r f; do cp "$f" "$pass/"; done)
rm -f "$out/voxelized"
touch "$out/snapshot-marker"
exit 0
