#!/bin/sh
# Prepares a round in $VOXEL_TRIALS, ~/voxel-trials by default. Each run's slot
# loses its files and skills and takes the skill the installed vxl prints.
# _runs/round.json records what the round runs on. A second skill file gives each
# prompt's -2 slot that skill in place of the installed vxl's, for comparing two
# skill texts.
# Usage: setup.sh [model] [effort] [second-skill]
set -eu
here=$(cd "$(dirname "$0")" && pwd)
root=${VOXEL_TRIALS:-$HOME/voxel-trials}
model=${1:-claude-opus-5-5}
effort=${2:-high}
second_skill=${3:-}

if [ -e "$root/_runs" ]; then
  echo "$root/_runs holds a round; move it aside with archive.sh first" >&2
  exit 1
fi

# Claude Code loads a user-level skill in place of a slot's skill of the same
# name.
user_skill=$HOME/.claude/skills/vxl-model/SKILL.md
if [ -n "$second_skill" ] && [ -e "$user_skill" ]; then
  echo "$user_skill would replace the second skill in each -2 slot; move it aside first" >&2
  exit 1
fi
if [ -e "$user_skill" ] && ! vxl integration skill vxl-model print | cmp -s - "$user_skill"; then
  echo "$user_skill differs from the installed vxl's skill and would replace each slot's" >&2
  echo "Reinstall it with \`vxl integration skill vxl-model install claude --user\` or move it aside first" >&2
  exit 1
fi

mkdir -p "$root/_harness" "$root/_runs"
cp "$here/snapshot.sh" "$root/_harness/snapshot.sh"
untrusted=0

for dir in $(jq -r '.[].dir' "$here/prompts.json"); do
  for run in "$dir" "$dir-2"; do
    slot=$root/$run
    rm -rf "$slot/.claude/skills"
    mkdir -p "$slot/.claude/skills/vxl-model"
    find "$slot" -mindepth 1 -maxdepth 1 ! -name .claude -exec rm -rf {} +
    if [ -n "$second_skill" ] && [ "$run" != "$dir" ]; then
      cp "$second_skill" "$slot/.claude/skills/vxl-model/SKILL.md"
    else
      vxl integration skill vxl-model print > "$slot/.claude/skills/vxl-model/SKILL.md"
    fi
    sed "s|@SNAPSHOT@|$root/_harness/snapshot.sh|" "$here/settings.json" > "$slot/.claude/settings.json"
    if ! jq -e --arg p "$slot" '.projects[$p].hasTrustDialogAccepted == true' "$HOME/.claude.json" > /dev/null; then
      untrusted=$((untrusted + 1))
    fi
  done
done

if git -C "$here" diff --quiet HEAD; then dirty=false; else dirty=true; fi
jq -n \
  --arg date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg commit "$(git -C "$here" rev-parse HEAD)" \
  --argjson dirty "$dirty" \
  --arg vxl "$(sed -n 's/^  vxl-version: "\(.*\)"$/\1/p' "$root/chair/.claude/skills/vxl-model/SKILL.md")" \
  --arg claude "$(claude --version | cut -d ' ' -f 1)" \
  --arg model "$model" \
  --arg effort "$effort" \
  --arg second "$(if [ -n "$second_skill" ]; then shasum -a 256 "$second_skill" | cut -d ' ' -f 1; fi)" \
  '{date: $date, commit: $commit, dirty: $dirty, vxlVersion: $vxl, claudeVersion: $claude, model: $model, effort: $effort}
    + if $second == "" then {} else {secondSkillSha256: $second} end' \
  > "$root/_runs/round.json"
cat "$root/_runs/round.json"

if [ "$untrusted" -gt 0 ]; then
  echo "$untrusted slots are untrusted, and a headless session ignores their settings." >&2
  echo "Run sh $here/trust.sh yourself to trust them." >&2
fi
