#!/bin/sh
# Prepares a round in $VOXEL_TRIALS, ~/voxel-trials by default. Slot 1 of each
# prompt runs arm 1, and its -2 slot runs arm 2. An arm's vxl defaults to the
# installed one and its skill to the one that vxl prints.
# Usage: setup.sh [--model <model>] [--effort <effort>] [--vxl-1 <binary>]
#   [--skill-1 <skill>] [--vxl-2 <binary>] [--skill-2 <skill>]
set -eu
here=$(cd "$(dirname "$0")" && pwd)
root=${VOXEL_TRIALS:-$HOME/voxel-trials}
model=claude-opus-5-5
effort=high
vxl_1= skill_1= vxl_2= skill_2=
while [ $# -gt 0 ]; do
  [ $# -ge 2 ] || { echo "$1 takes a value" >&2; exit 1; }
  case $1 in
    --model) model=$2 ;;
    --effort) effort=$2 ;;
    --vxl-1) vxl_1=$2 ;;
    --skill-1) skill_1=$2 ;;
    --vxl-2) vxl_2=$2 ;;
    --skill-2) skill_2=$2 ;;
    *) echo "$1 is no option of setup.sh" >&2; exit 1 ;;
  esac
  shift 2
done

if [ -e "$root/_runs" ]; then
  echo "$root/_runs holds a round; move it aside with archive.sh first" >&2
  exit 1
fi

sha() { find "$1" -type f | sort | xargs shasum -a 256 | sed "s| $1/| |" | shasum -a 256 | cut -d ' ' -f 1; }

# Each arm's binary and skill land in _harness/arm-<n>. run-trial.sh puts the
# binary first on the slot's PATH, so a later install leaves the round alone.
mkdir -p "$root/_harness"
cp "$here/snapshot.sh" "$root/_harness/snapshot.sh"
arms=
for n in 1 2; do
  eval "vxl=\$vxl_$n skill=\$skill_$n"
  arm=$root/_harness/arm-$n
  rm -rf "$arm"
  mkdir -p "$arm/bin" "$arm/skill"
  binary=${vxl:-$(command -v vxl)} || { echo "no vxl installed" >&2; exit 1; }
  [ -f "$binary" ] && [ -x "$binary" ] || { echo "$binary is no vxl binary" >&2; exit 1; }
  binary=$(cd "$(dirname "$binary")" && pwd)/$(basename "$binary")
  [ -z "$skill" ] || skill=$(cd "$(dirname "$skill")" && pwd)/$(basename "$skill")
  cp "$binary" "$arm/bin/vxl"
  if [ -d "$skill" ]; then
    [ -f "$skill/SKILL.md" ] || { echo "$skill holds no SKILL.md" >&2; exit 1; }
    cp -R "$skill/." "$arm/skill/"
  elif [ -n "$skill" ]; then
    cp "$skill" "$arm/skill/SKILL.md"
  else
    "$arm/bin/vxl" integration skill vxl-model print > "$arm/skill/SKILL.md"
  fi
  arms=$arms$(jq -n \
    --arg n "$n" \
    --arg vxl "$binary" \
    --arg version "$("$arm/bin/vxl" --version | cut -d ' ' -f 2)" \
    --arg vxlSha "$(shasum -a 256 "$arm/bin/vxl" | cut -d ' ' -f 1)" \
    --arg skill "${skill:-printed}" \
    --arg skillVersion "$(sed -n 's/^  vxl-version: "\(.*\)"$/\1/p' "$arm/skill/SKILL.md")" \
    --arg skillSha "$(sha "$arm/skill")" \
    '{($n): {vxl: $vxl, vxlVersion: $version, vxlSha256: $vxlSha, skill: $skill, skillVxlVersion: $skillVersion, skillSha256: $skillSha}}')
done

# Claude Code loads a user-level skill in place of a slot's skill of the same
# name.
user_skill=$HOME/.claude/skills/vxl-model
if [ -e "$user_skill" ]; then
  for n in 1 2; do
    if ! diff -rq "$user_skill" "$root/_harness/arm-$n/skill" > /dev/null; then
      echo "$user_skill differs from arm $n's skill and would replace it; move it aside first" >&2
      exit 1
    fi
  done
fi

mkdir "$root/_runs"
untrusted=0
for dir in $(jq -r '.[].dir' "$here/prompts.json"); do
  for n in 1 2; do
    run=$dir
    [ "$n" = 1 ] || run=$dir-2
    slot=$root/$run
    rm -rf "$slot/.claude/skills"
    mkdir -p "$slot/.claude/skills/vxl-model"
    find "$slot" -mindepth 1 -maxdepth 1 ! -name .claude -exec rm -rf {} +
    cp -R "$root/_harness/arm-$n/skill/." "$slot/.claude/skills/vxl-model/"
    sed "s|@SNAPSHOT@|$root/_harness/snapshot.sh|" "$here/settings.json" > "$slot/.claude/settings.json"
    if ! jq -e --arg p "$slot" '.projects[$p].hasTrustDialogAccepted == true' "$HOME/.claude.json" > /dev/null; then
      untrusted=$((untrusted + 1))
    fi
  done
done

if git -C "$here" diff --quiet HEAD; then dirty=false; else dirty=true; fi
printf '%s' "$arms" | jq -s \
  --arg date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg commit "$(git -C "$here" rev-parse HEAD)" \
  --argjson dirty "$dirty" \
  --arg claude "$(claude --version | cut -d ' ' -f 1)" \
  --arg model "$model" \
  --arg effort "$effort" \
  '{date: $date, commit: $commit, dirty: $dirty, claudeVersion: $claude, model: $model, effort: $effort, arms: add}' \
  > "$root/_runs/round.json"
cat "$root/_runs/round.json"

if [ "$untrusted" -gt 0 ]; then
  echo "$untrusted slots are untrusted, and a headless session ignores their settings." >&2
  echo "Run sh $here/trust.sh yourself to trust them." >&2
fi
