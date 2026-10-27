#!/bin/sh
# Marks every run's slot trusted in ~/.claude.json. A headless session ignores
# the settings of an untrusted directory. A person runs this, never an agent.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
root=${VOXEL_TRIALS:-$HOME/voxel-trials}
slots=$(jq --arg root "$root" '[.[].dir | ($root + "/" + .), ($root + "/" + . + "-2")]' "$here/prompts.json")
jq --argjson slots "$slots" \
  'reduce $slots[] as $s (.; .projects[$s] = ((.projects[$s] // {}) + {hasTrustDialogAccepted: true}))' \
  "$HOME/.claude.json" > "$HOME/.claude.json.trials"
mv "$HOME/.claude.json.trials" "$HOME/.claude.json"
