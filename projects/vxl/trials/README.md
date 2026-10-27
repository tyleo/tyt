# Voxel-modeling trials

The trials run each prompt in `prompts.json` through the vxl-model skill twice,
each time in a fresh headless Claude Code session. The voxel-modeling plan's
[trials](../../../doc/plan/open/voxel-modeling/trials.md) log the first
round.

## Layout

The trials run outside the repository because a session inside it could read
the builder and the docs in place of the skill. `$VOXEL_TRIALS`,
`~/voxel-trials` by default, holds a round:

1. `<directory>` and `<directory>-2`, the slots the two runs of a prompt work in
2. `_harness/snapshot.sh`, the hook that copies each pass's renders, model
   file, and report into `_runs/<run>/passes`
3. `_runs`, the round's `round.json` and one record per run
4. `findings`, the round's ranked findings and the images they show
5. `gallery`, the page `build-gallery.py` writes
6. `rounds/<round>`, each round `archive.sh` moved aside

A headless session ignores the settings of an untrusted directory. The slots
stay in place from round to round, and `trust.sh` marks them trusted once. A
person runs `trust.sh` because it edits `~/.claude.json`.

Each slot's settings run its session in auto mode. Claude Code then approves any
command a classifier judges safe.

## A round

```sh
npm run vxl:install
sh projects/vxl/trials/setup.sh
python3 projects/vxl/trials/build-gallery.py
sh projects/vxl/trials/archive.sh 2026-10-04
```

1. The commit under test is checked out, and `npm run vxl:install` installs its
   vxl
2. `setup.sh` clears the slots, prints the installed vxl's skill into each, and
   records the round. It takes the model and the effort, `claude-opus-5-5` and
   `high` by default. A `vxl-model` skill under `~/.claude/skills` replaces each
   slot's copy, and `setup.sh` errors when that skill differs from the installed
   vxl's. A skill file as the third argument goes into each `-2` slot in place
   of the installed vxl's, so a round can compare two skill texts
3. A Claude Code session in this repository runs `workflow.js` with the
   contents of `prompts.json` as its `trials` argument. The workflow runs 12
   sessions at a time, analyzes both runs of each prompt, and synthesizes the
   findings. `only` limits the round to some directories, and `concurrency`
   sets how many sessions run at once
4. `build-gallery.py` writes the gallery. `--findings <dir>` adds the ranked
   findings in the folder's `findings.json`. Each finding lists the ids of the
   analysis items behind it. The build errors when an item sits in no finding.
   `analysis_items.py` prints the items with their ids
5. `archive.sh <round>` moves the round to `rounds/<round>` and copies each
   session's full transcript beside its record. It moves any memory a session
   wrote there too because the next session in the slot would load it

`run-trial.sh <run>` runs one session by hand, and `wait-trial.sh <run>` waits
for it. `summarize-transcript.py <run>` prints a run's transcript without its
images, and `errors.sh <run>` lists the run's failed tool calls. Both take
`--round <round>` for an archived round.

## Exact reruns

`round.json` records the date, the repository commit, and whether the tree held
local edits. It also records the vxl version in the skill, the Claude Code
version, the model, and the effort. `run-trial.sh` passes the model and the
effort to every session.

A rerun checks out the recorded commit and installs its vxl. It installs the
recorded Claude Code with `claude install <version>` and runs `setup.sh` with
the recorded model and effort. The rerun repeats the conditions. The sessions
still sample, and their models differ from run to run.
