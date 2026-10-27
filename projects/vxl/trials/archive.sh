#!/bin/sh
# Moves the round in $VOXEL_TRIALS aside to rounds/<round>, with each session's
# full transcript and any memory it wrote. Each slot keeps its .claude folder for
# the next round.
# Usage: archive.sh <round>
set -eu
here=$(cd "$(dirname "$0")" && pwd)
root=${VOXEL_TRIALS:-$HOME/voxel-trials}
dest=$root/rounds/$1
[ -d "$root/_runs" ] || { echo "no round in $root" >&2; exit 1; }
[ ! -e "$dest" ] || { echo "$dest exists" >&2; exit 1; }
mkdir -p "$dest/slots"
cp "$root/chair/.claude/skills/vxl-model/SKILL.md" "$dest/SKILL.md"
if ! cmp -s "$root/chair/.claude/skills/vxl-model/SKILL.md" "$root/chair-2/.claude/skills/vxl-model/SKILL.md"; then
  cp "$root/chair-2/.claude/skills/vxl-model/SKILL.md" "$dest/SKILL-2.md"
fi

for dir in $(jq -r '.[].dir' "$here/prompts.json"); do
  for run in "$dir" "$dir-2"; do
    slot=$root/$run
    mkdir -p "$dest/slots/$run"
    find "$slot" -mindepth 1 -maxdepth 1 ! -name .claude -exec mv {} "$dest/slots/$run/" \;
    result=$root/_runs/$run/result.json
    [ -s "$result" ] || continue
    id=$(jq -r .session_id "$result")
    project=$HOME/.claude/projects/$(printf '%s' "$slot" | sed 's/[^A-Za-z0-9]/-/g')
    [ -f "$project/$id.jsonl" ] && cp "$project/$id.jsonl" "$root/_runs/$run/session.jsonl"
    [ -d "$project/$id" ] && cp -R "$project/$id" "$root/_runs/$run/session"
    if [ -n "$(ls -A "$project/memory" 2>/dev/null)" ]; then
      mv "$project/memory" "$root/_runs/$run/memory"
    fi
  done
done

mv "$root/_runs" "$dest/runs"
for folder in findings gallery; do
  if [ -d "$root/$folder" ]; then mv "$root/$folder" "$dest/$folder"; fi
done
echo "$dest"
