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
   file, and report into `_runs/<run>/passes`, and `_harness/arm-<n>`, the
   vxl binary and skill of each arm
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
2. `setup.sh` clears the slots, gives each its arm, and records the round. A
   `vxl-model` skill under `~/.claude/skills` replaces each slot's copy, and
   `setup.sh` errors when that skill differs from an arm's
3. A Claude Code session in this repository runs `workflow.js` with the
   contents of `prompts.json` as its `trials` argument. The workflow runs 12
   sessions at a time, analyzes both runs of each prompt, and synthesizes the
   findings. Its agents run on Opus even when the session runs another model.
   `only` limits the round to some directories, and `concurrency` sets how many
   sessions run at once
4. A judge ranks each prompt's final models, which `blind-views.py` renders
   with the installed vxl under shuffled labels. A shot agent renders the
   close-ups and inside views a prompt lists under `shots` in `prompts.json`,
   once for each model. The judge sees no transcript or skill. It ranks once
   with each label in each listed position. With `reference` set to an
   archived round, the judge ranks that round's runs too. `round` judges an
   archived round alone. `judge-stats.py` unblinds the rankings and tests one
   arm against the other over the prompts. `process-metrics.py` reads each
   run's transcript and files and compares the arms on cost, time, failed
   commands, and the checks left open. `build-viewer.py` writes a page that
   shows the judged models one view at a time for an optional ranking by eye
5. `build-gallery.py` writes the gallery. `--findings <dir>` adds the ranked
   findings in the folder's `findings.json`. Each finding lists the ids of the
   analysis items behind it. The build errors when an item sits in no finding.
   `analysis_items.py` prints the items with their ids
6. `archive.sh <round>` moves the round and its arms to `rounds/<round>` and
   copies each session's full transcript beside its record. It moves any memory a session
   wrote there too because the next session in the slot would load it

`run-trial.sh <run>` runs one session by hand, and `wait-trial.sh <run>` waits
for it. `summarize-transcript.py <run>` prints a run's transcript without its
images, and `errors.sh <run>` lists the run's failed tool calls. Both take
`--round <round>` for an archived round.

## Arms

Slot 1 of each prompt runs arm 1, and the `-2` slot runs arm 2. An arm holds a
vxl binary, a skill, a model, and an effort:

| Flag           | Sets                                 | Default            |
| -------------- | ------------------------------------ | ------------------ |
| `--vxl-<n>`    | The binary first on the arm's PATH   | The installed vxl  |
| `--skill-<n>`  | A `SKILL.md` or a folder holding one | The binary's skill |
| `--model-<n>`  | The model of the arm's sessions      | `claude-opus-5-5`  |
| `--effort-<n>` | The effort of the arm's sessions     | `high`             |

With every flag on its default, the round runs one arm twice. A skill change
sets `--skill-2`, a tool change sets `--vxl-2` to the candidate build, and a
model comparison sets `--model-2`. A second round swaps the arms to cancel any
lean toward a slot.

## Exact reruns

`round.json` records the date, the repository commit, and whether the tree held
local edits. It also records the Claude Code version and each arm's source,
version, hashes, model, and effort. `run-trial.sh` passes each session its arm's
model and effort.

A rerun checks out the recorded commit and installs its vxl. It installs the
recorded Claude Code with `claude install <version>` and runs `setup.sh` with
each arm's recorded model and effort. An archived round keeps its arms in
`arm-1` and `arm-2` for `--vxl-<n>` and `--skill-<n>`. The rerun repeats the
conditions. The sessions still sample, and their models differ from run to run.
