export const meta = {
  name: 'voxel-trials',
  description: 'Run each voxel-modeling trial prompt twice in headless Claude Code sessions, then analyze and compare the runs',
  whenToUse: 'A round of the voxel-modeling trials, after setup.sh in projects/vxl/trials',
  phases: [
    { title: 'Run', model: 'opus', detail: 'one headless claude -p session per trial directory, 12 at a time' },
    { title: 'Analyze', model: 'opus', detail: 'one agent per prompt reads both runs and compares them' },
    { title: 'Judge', model: 'opus', detail: 'each prompt\'s final models and shots, rendered under shuffled labels and ranked once per label position' },
    { title: 'Synthesize', model: 'opus', detail: 'cross-trial patterns for the Phase 2 design' },
  ],
}

// Paths read from the repository root because workflow agents start there.
const HARNESS = 'projects/vxl/trials'
// Every agent runs on this model even when the launching session runs another.
const MODEL = 'opus'
const LIMIT = (args && args.concurrency) || 12
const REFERENCE = args && args.reference

let active = 0
const waiters = []
async function acquire() {
  if (active < LIMIT) { active++; return }
  await new Promise(resolve => waiters.push(resolve))
}
function release() {
  const next = waiters.shift()
  if (next) next()
  else active--
}

const RUN_SCHEMA = {
  type: 'object',
  properties: {
    launched: { type: 'boolean' },
    exitCode: { type: 'integer' },
    passes: { type: 'integer' },
    minutes: { type: 'number' },
    note: { type: 'string', description: 'the head of stderr.log, empty when stderr is empty' },
  },
  required: ['launched', 'exitCode', 'passes', 'minutes', 'note'],
}

function runPrompt(runDir) {
  return `You babysit one headless voxel-modeling trial run whose working directory is ~/voxel-trials/${runDir}. You only launch it and wait for it. Never read, edit, create, or delete anything under ~/voxel-trials/${runDir}, never kill or signal a process, and never help the trial.

1. Launch the run detached from the repository root, with exactly this Bash command: nohup sh ${HARNESS}/run-trial.sh ${runDir} > /dev/null 2>&1 &
2. Run \`sh ${HARNESS}/wait-trial.sh ${runDir}\` with the Bash tool's timeout parameter set to 600000. The script blocks up to nine minutes, then prints "running <m> min, <n> passes" or "done <exit code>". Run it again after every "running" line until it prints "done". A run can take up to two hours. Do not use sleep or any other way to wait.
3. Run \`ls ~/voxel-trials/_runs/${runDir}/passes 2>/dev/null | wc -l; cat ~/voxel-trials/_runs/${runDir}/started ~/voxel-trials/_runs/${runDir}/finished; head -c 600 ~/voxel-trials/_runs/${runDir}/stderr.log\`.

Return launched=true, the exit code from the "done" line, the passes count, the minutes between the started and finished stamps, and the stderr head as note.`
}

async function runOnce(runDir) {
  await acquire()
  try {
    return await agent(runPrompt(runDir), {
      label: `run:${runDir}`, phase: 'Run', model: MODEL, effort: 'low', schema: RUN_SCHEMA,
    })
  } finally {
    release()
  }
}

async function runTrial(runDir) {
  let result = await runOnce(runDir)
  if (result && result.exitCode !== 0 && result.passes === 0) {
    log(`${runDir} exited ${result.exitCode} with no passes (${result.note.slice(0, 120)}); relaunching once`)
    result = await runOnce(runDir)
  }
  log(`${runDir} done: exit ${result ? result.exitCode : 'unknown'}, ${result ? result.passes : '?'} passes, ${result ? Math.round(result.minutes) : '?'} min`)
  return result
}

const RUN_ANALYSIS = {
  type: 'object',
  properties: {
    run: { type: 'integer', description: '1 or 2' },
    skillLoaded: { type: 'boolean', description: 'the session loaded the vxl-model skill' },
    finished: { type: 'boolean', description: 'the session called the model done, rather than stopping on an error, a question, or the time cap' },
    passes: { type: 'integer', description: 'the PASS markers in the transcript summary' },
    failedPasses: { type: 'integer', description: 'passes whose command exited with an error' },
    minutes: { type: 'number' },
    turns: { type: 'integer' },
    costUsd: { type: 'number' },
    voxelSize: { type: 'string', description: 'the final voxel size or resolution flag' },
    voxels: { type: 'integer', description: 'voxel count in the final report' },
    pieces: { type: 'integer', description: 'piece count in the final report' },
    usedParts: { type: 'boolean' },
    verdict: { type: 'string', enum: ['good', 'fair', 'poor', 'failed'], description: 'the grade the rubric in the prompt gives the final renders' },
    assessment: { type: 'string', description: 'two to four sentences on how well the final renders match the prompt, naming what reads right and what reads wrong' },
    caption: { type: 'string', description: 'one short sentence for a gallery card' },
    progress: { type: 'string', description: 'one sentence on how the model changed from the first pass to the last' },
    failures: { type: 'array', items: { type: 'string' }, description: 'errors, broken passes, and wrong turns, each with its pass number and the error text when short' },
    operationsLacked: { type: 'array', items: { type: 'string' }, description: 'operations the session wanted that the API lacks, each with what the session did instead' },
    skillMisses: { type: 'array', items: { type: 'string' }, description: 'API features or skill advice the session missed or misused although SKILL.md has them, where the miss cost the model quality or cost passes' },
    reportDataLacked: { type: 'array', items: { type: 'string' }, description: 'report or render data the session wanted and lacked' },
    flatColors: { type: 'string', description: 'where the colors read flat or wrong in the renders, or "none"' },
    workarounds: { type: 'array', items: { type: 'string' } },
    renders: {
      type: 'object',
      properties: {
        sheet: { type: 'string', description: 'the review sheet, which tiles the hero, front, right, back, and top views' },
        extra: { type: 'array', items: { type: 'string' }, description: 'any other final PNGs the session rendered, such as cutaways' },
      },
      required: ['sheet', 'extra'],
      description: 'absolute paths of the final renders; empty strings when missing',
    },
    modelFile: { type: 'string', description: 'absolute path of the final model file' },
  },
  required: ['run', 'skillLoaded', 'finished', 'passes', 'failedPasses', 'minutes', 'turns', 'costUsd', 'voxelSize', 'voxels', 'pieces', 'usedParts', 'verdict', 'assessment', 'caption', 'progress', 'failures', 'operationsLacked', 'skillMisses', 'reportDataLacked', 'flatColors', 'workarounds', 'renders', 'modelFile'],
}

const TRIAL_ANALYSIS = {
  type: 'object',
  properties: {
    num: { type: 'integer' },
    dir: { type: 'string' },
    runs: { type: 'array', items: RUN_ANALYSIS, minItems: 2, maxItems: 2 },
    comparison: { type: 'string', description: 'two to four sentences on how the two runs differed in approach, passes, and result' },
    stressFindings: { type: 'string', description: 'how each run fared against each point of the stress note, or "" when the trial has none' },
    better: { type: 'string', enum: ['1', '2', 'tie'] },
  },
  required: ['num', 'dir', 'runs', 'comparison', 'stressFindings', 'better'],
}

const RUBRIC = `Grade each run's final renders with this rubric:
1. good: every element the prompt names is present and reads at first glance in the hero view, with believable proportions and colors and no defect the prompt's writer would notice
2. fair: every named element is present, but one or two read weakly, such as a hidden or tiny detail, an off proportion, a flat or wrong color, or a visible defect
3. poor: a named element is missing or unrecognizable, or the subject reads as something else
4. failed: the session left no final model or renders`

function analysisPrompt(t, runResults) {
  const runs = [t.dir, `${t.dir}-2`]
  return `You analyze voxel-modeling trial ${t.num}. The prompt "${t.prompt}" ran twice, each time in a fresh headless Claude Code session that had only the vxl-model skill to go on. Run 1 worked in ~/voxel-trials/${runs[0]} and run 2 in ~/voxel-trials/${runs[1]}. The harness recorded each run in ~/voxel-trials/_runs/<run directory>: result.json holds the session's final message, turns, duration, cost, permission denials, and model usage; stderr.log holds stderr; passes/NN/ holds the renders and model file each pass left. The harness reported: ${JSON.stringify(runResults)}.

This job is read-only. Never edit, create, or delete anything under ~/voxel-trials except the files this prompt tells you to write.

For each run:
1. Run \`python3 ${HARNESS}/summarize-transcript.py <run directory> > ~/voxel-trials/_runs/<run directory>/transcript.txt\` from the repository root, then read transcript.txt in full, in chunks with Read offsets when it is long. It holds the session's messages, commands, model edits, and reports, with images left out.
2. Read the final model file. Look at the final renders with Read (the review sheet, which tiles the hero, front, right, back, and top views, and any other PNG the session rendered last), and at the sheet in passes/01 to see how far the model came.
3. Each run followed the skill in its slot, ~/voxel-trials/<run directory>/.claude/skills/vxl-model/SKILL.md. Arm 1 in ~/voxel-trials/_runs/round.json gives run 1's vxl version, skill, model, and effort, and arm 2 gives run 2's. The two runs can differ in any of them. Credit a difference the arms explain to the arm, not the session. Before listing an operation as lacking, confirm in that run's SKILL.md that the API lacks it. When the API has it and the session missed it, list it under skillMisses instead.
${t.stress ? `\nThe prompt was chosen to stress this: ${t.stress}. Report how each run fared against each point.\n` : ''}${t.check ? `\n${t.check} Stay read-only.\n` : ''}
${RUBRIC}

Judge the model, not the process. A session may use any tools in any order, and a skipped skill step counts only where the model or the session's time suffered. Judge each result from the renders the way the person who wrote the prompt would. Be concrete and critical, and keep each list item to one or two sentences. Then write the whole object you return as JSON to ~/voxel-trials/_runs/${t.dir}/analysis.json and return it.`
}

// The judge sees each prompt's final models under shuffled labels, and the
// reference round's too when the caller passes one. `round` judges an archived
// round without running or analyzing anything.
const ROUND = args && args.round
const SOURCES = REFERENCE ? ['1', '2', 'r1', 'r2'] : ['1', '2']
const NAMES = { 1: 'run 1', 2: 'run 2', r1: 'reference 1', r2: 'reference 2' }
const LABELS = SOURCES.map((_, i) => 'ABCD'[i])
const RUNS_DIR = ROUND ? `~/voxel-trials/rounds/${ROUND}/runs` : '~/voxel-trials/_runs'

function permutations(items) {
  if (items.length <= 1) return [items]
  return items.flatMap((item, i) =>
    permutations([...items.slice(0, i), ...items.slice(i + 1)]).map(rest => [item, ...rest]))
}
const ORDERS = permutations(SOURCES)

const COPY_SCHEMA = {
  type: 'object',
  properties: { listing: { type: 'string', description: 'the script output, unchanged' } },
  required: ['listing'],
}

const JUDGE_SCHEMA = {
  type: 'object',
  properties: {
    ranks: {
      type: 'object',
      properties: Object.fromEntries(LABELS.map(label => [label, { type: 'integer', minimum: 1, maximum: LABELS.length }])),
      required: LABELS,
      description: 'each label\'s rank, 1 for the best, with models you cannot tell apart sharing a rank',
    },
    reasons: { type: 'string', description: 'two to four sentences on what sets the ranks' },
  },
  required: ['ranks', 'reasons'],
}

const SHOT_SCHEMA = {
  type: 'object',
  properties: {
    shots: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          shown: { type: 'boolean', description: 'false when the model has no such feature to show' },
          note: { type: 'string', description: 'one sentence on what the shot shows, or on what the model lacks' },
        },
        required: ['shown', 'note'],
      },
    },
  },
  required: ['shots'],
}

// A shot agent renders a prompt's shots of one label's model without learning
// which run made it. Every label gets the same instructions.
function shotPrompt(t, label) {
  const judgeDir = `${RUNS_DIR}/${t.dir}/judge`
  const models = `${judgeDir}/models/${label}`
  const shots = t.shots.map((shot, i) => `${i + 1}. ${shot}`).join('\n')
  return `You render camera shots of a voxel model made from the prompt "${t.prompt}" so a judge can compare it with other models of the same prompt. ${models} holds model-<n>.voxj documents with their TypeScript model files and voxelize reports. Read the reports and model files to learn where each part sits, in meters. Never edit them.

The shots:
${shots}

For each shot k, render it from the .voxj that holds its subject with \`vxl object render <voxj> --lights-from review --bloom-strength 1 --to png --file-stem shot-<k>\` and the view flags it takes, such as --view-orbit, --view-look-at, --view-select, --view-frame world with --view-position, --view-fov, or --rotate. \`vxl object render --help\` lists them. The PNG lands beside the .voxj. Look at it with Read and adjust, up to 4 renders a shot, until the subject fills the frame and nothing hides it. Then copy the last render to ${judgeDir}/${label}-shot-<k>.png. When the model has no such feature, render nothing for that shot and return shown=false.

Frame each shot plainly, as a photographer would show the feature to its buyer. A flattering or unflattering angle misleads the judge. When every shot is done, delete ${models}. Return one entry per shot in order.`
}

// Each judging rotates the label listing by one place. Every model then sits in
// every position once.
async function judge(t) {
  const order = ORDERS[(t.num * 7) % ORDERS.length]
  const flags = `${ROUND ? ` --round ${ROUND}` : ''}${REFERENCE ? ` --reference ${REFERENCE}` : ''}`
  const copied = await agent(
    `Run \`python3 ${HARNESS}/blind-views.py ${t.dir} ${order.join(',')}${flags}\` from the repository root and return its output unchanged as listing. Do nothing else.`,
    { label: `blind:${t.dir}`, phase: 'Judge', model: MODEL, effort: 'low', schema: COPY_SCHEMA },
  )
  const lines = Object.fromEntries(copied.listing.trim().split('\n').map(line => [line[0], line]))
  const missing = LABELS.filter(label => !lines[label])
  if (missing.length) throw new Error(`blind-views listed no ${missing.join(', ')}: ${copied.listing}`)
  if (t.shots) {
    await Promise.all(LABELS.filter(label => !lines[label].endsWith('no model')).map(async label => {
      const result = await agent(shotPrompt(t, label), { label: `shots:${t.dir}:${label}`, phase: 'Judge', model: MODEL, effort: 'medium', schema: SHOT_SCHEMA })
      if (!result || result.shots.length !== t.shots.length) throw new Error(`shots of ${t.dir} ${label} returned ${JSON.stringify(result)}`)
      const shots = result.shots.map((shot, i) => shot.shown
        ? `${RUNS_DIR}/${t.dir}/judge/${label}-shot-${i + 1}.png`
        : `shot ${i + 1} not modeled`)
      lines[label] += `, ${shots.join(', ')}`
    }))
  }
  const verdicts = await Promise.all(LABELS.map((_, k) => {
    const listing = LABELS.map((_, i) => lines[LABELS[(i + k) % LABELS.length]]).join('\n')
    return agent(`You judge ${LABELS.length} voxel models made from the prompt "${t.prompt}". Each label holds one run's renders. <label>-sheet-<n>.png is a review sheet of one of its models, tiling the hero, front, right, back, and top views. ${t.shots ? `<label>-shot-<k>.png shows shot k of every model: ${t.shots.map((shot, i) => `${i + 1}. ${shot}`).join('; ')}. "shot k not modeled" marks a model with no such feature. ` : ''}A label with no files left no model.

${listing}

Look at every listed file with Read, and read nothing else. Rank the models as the prompt's writer would, 1 for the best:
1. Every element the prompt names is present and reads at first glance
2. Proportions, scale, and colors are believable
3. No part floats, breaks apart, or shows a visible defect

A finer voxel grid earns no credit. Models you cannot tell apart share a rank. Write the object you return as JSON to ${RUNS_DIR}/${t.dir}/judge/verdict-${k + 1}.json too.`,
      { label: `judge:${t.dir}:${k + 1}`, phase: 'Judge', model: MODEL, effort: 'high', schema: JUDGE_SCHEMA },
    )
  }))
  return verdicts.map(verdict => verdict && {
    ranks: Object.fromEntries(LABELS.map(label => [NAMES[order[LABELS.indexOf(label)]], verdict.ranks[label]])),
    reasons: verdict.reasons,
  })
}

async function trialPipeline(t, runPromises) {
  const results = await Promise.all(runPromises)
  const [analysis, judgment] = await Promise.all([
    agent(analysisPrompt(t, results), {
      label: `analyze:${t.dir}`, phase: 'Analyze', model: MODEL, effort: 'high', schema: TRIAL_ANALYSIS,
    }).catch(e => { log(`analysis of ${t.dir} failed: ${e}`); return null }),
    judge(t).catch(e => { log(`judging ${t.dir} failed: ${e}`); return null }),
  ])
  return { dir: t.dir, analysis, judgment }
}

const LOAD_SCHEMA = {
  type: 'object',
  properties: {
    trials: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          num: { type: 'integer' }, dir: { type: 'string' }, prompt: { type: 'string' },
          stress: { type: 'string' }, check: { type: 'string' },
          shots: { type: 'array', items: { type: 'string' } },
        },
        required: ['num', 'dir', 'prompt'],
      },
    },
  },
  required: ['trials'],
}

// The caller passes prompts.json as args.trials for an exact copy. An agent
// reads it otherwise.
const listed = (args && args.trials) || (await agent(
  `Read ${HARNESS}/prompts.json from the repository root and return every entry unchanged, character for character, as trials. Do not edit anything.`,
  { label: 'load prompts', phase: 'Run', model: MODEL, effort: 'low', schema: LOAD_SCHEMA },
)).trials
const only = args && args.only ? new Set(args.only) : null
const trials = listed.filter(t => !only || only.has(t.dir))

if (ROUND) {
  phase('Judge')
  const judged = await Promise.all(trials.map(t =>
    judge(t).catch(e => { log(`judging ${t.dir} failed: ${e}`); return null })))
  return { judgments: trials.map((t, i) => ({ dir: t.dir, verdicts: judged[i] })) }
}

phase('Run')
const firstRuns = trials.map(t => runTrial(t.dir))
const secondRuns = trials.map(t => runTrial(`${t.dir}-2`))
const outcomes = await Promise.all(trials.map((t, i) => trialPipeline(t, [firstRuns[i], secondRuns[i]])))
const analyses = outcomes.map(o => o.analysis).filter(Boolean)
const judgments = outcomes.map(o => ({ dir: o.dir, verdicts: o.judgment }))

phase('Synthesize')
const SYNTHESIS = {
  type: 'object',
  properties: {
    operationsLacked: { type: 'array', items: { type: 'object', properties: { item: { type: 'string' }, trials: { type: 'array', items: { type: 'integer' } }, note: { type: 'string' } }, required: ['item', 'trials', 'note'] } },
    recurringFailures: { type: 'array', items: { type: 'object', properties: { item: { type: 'string' }, trials: { type: 'array', items: { type: 'integer' } }, note: { type: 'string' } }, required: ['item', 'trials', 'note'] } },
    skillMisses: { type: 'array', items: { type: 'object', properties: { item: { type: 'string' }, trials: { type: 'array', items: { type: 'integer' } }, note: { type: 'string' } }, required: ['item', 'trials', 'note'] } },
    reportDataLacked: { type: 'array', items: { type: 'object', properties: { item: { type: 'string' }, trials: { type: 'array', items: { type: 'integer' } }, note: { type: 'string' } }, required: ['item', 'trials', 'note'] } },
    colorIssues: { type: 'array', items: { type: 'object', properties: { item: { type: 'string' }, trials: { type: 'array', items: { type: 'integer' } }, note: { type: 'string' } }, required: ['item', 'trials', 'note'] } },
    consistency: { type: 'string', description: 'how much the two runs of a prompt varied, with examples' },
    worked: { type: 'string', description: 'what the loop and the skill did well' },
    phase2: { type: 'array', items: { type: 'string' }, description: 'ranked recommendations for the Phase 2 design, each tied to trial numbers' },
  },
  required: ['operationsLacked', 'recurringFailures', 'skillMisses', 'reportDataLacked', 'colorIssues', 'consistency', 'worked', 'phase2'],
}
const synthesis = await agent(`You synthesize the voxel-modeling trials. ${trials.length} prompts, from single props to scenes and interiors, each ran twice in fresh headless Claude Code sessions with the vxl-model skill (~/voxel-trials/chair/.claude/skills/vxl-model/SKILL.md, and ~/voxel-trials/chair-2/.claude/skills/vxl-model/SKILL.md for every second run). Arm 1 in ~/voxel-trials/_runs/round.json gives every first run's vxl version, skill, model, and effort, and arm 2 gives every second run's. The two can differ in any of them. Per-trial analyses follow as JSON. Merge duplicates across trials, rank each list by how many trials it touched and how badly, and cite trial numbers. Recommendations for Phase 2 cover operators, patterns, report data, render support, skill text, and the .sdfj format. Spot-check any claim you rely on heavily by reading ~/voxel-trials/_runs/<directory>/analysis.json or the transcript.txt beside it. Write the object you return as JSON to ~/voxel-trials/_runs/synthesis.json too.

${JSON.stringify(analyses)}`, { label: 'synthesize', phase: 'Synthesize', model: MODEL, effort: 'high', schema: SYNTHESIS })

return { analyses, judgments, synthesis }