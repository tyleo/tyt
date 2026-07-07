# Continue the ty-math adoption

You are consolidating hand-rolled math in the voxsmith converters onto the
`ty-math` crate, from an audit of `projects/utilities/voxsmith/src/convert`. The
work is three tracks: harmless cleanups with no new API, a `ty-math` extension with
adoption, and an investigation of the heavier `internal/` logic. One reviewable,
staged chunk per session, then stop for review.

## Orient first, every session

1. Read `doc/plan/closed/ty-math-adoption/README.md` for the three tracks, the vmax
   isolation rule, and the decisions (Q1 extend freely, Q2 isolate vmax, Q3 method
   names).
2. Read `doc/plan/closed/ty-math-adoption/checklist.md` for the per-track items with
   line numbers, and
   `doc/plan/closed/ty-math-adoption/reference/implementation-decisions.md` for the
   code-level choices made so far.
3. Run `git log --oneline -15` and `git status`. Each track is its own branch off
   `main`; branch if you are on `main` and starting a track. Confirm the audit line
   numbers still hold before editing.

## Pick the work

- Take the first unchecked `[ ]` item in the track you are advancing. The tracks
  are loosely ordered (A needs no API, B extends it, C investigates), but each is a
  standalone branch; do not interleave two in one session.
- Within a track, do non-vmax items first. Any edit under `convert/vmax` or
  `internal/vmax` goes in its own trailing commit, because a second branch is
  editing vmax (Q2). Do not mix vmax and non-vmax files in one staged chunk.
- Track C is investigation first: read, catalog, adopt what fits, and file larger
  primitives as new checklist items rather than building them inline.
- State in one line which track and item you are doing.

## Do the work

- Follow `CLAUDE.md`: Rust edition 2024, consolidated nested `use`, one public type
  per file in snake_case, doc comments on public items, comments wrapped to 80
  columns and ASCII-only.
- Track A and Track B change no serialized bytes. If a converter golden or a
  round-trip test changes output, stop; the refactor is not behavior-preserving.
- For a `ty-math` addition, put it in the file's existing float-macro form with a
  doc comment and a unit test in that file's `tests` module. No version bump is
  needed; the workspace patch carries the new methods to every consumer.
- Record any non-obvious code-level decision, including a confirmed method name, in
  `reference/implementation-decisions.md`, and check off the items you complete.
- Verify before staging: `cargo check`, then `cargo fmt --all`, then
  `cargo clippy --workspace --all-targets -- -D warnings`, then `cargo test -p` the
  crate or crates you touched (`ty-math` and `voxsmith`).

## Stage, do not commit

- `git add` everything you changed, including the checklist and decision-log edits.
  Do not `git commit`, `git push`, or amend.
- Then stop and present for review:
  1. A short summary of what changed and why.
  2. The files touched, grouped logically, and whether vmax is isolated.
  3. Test and lint results.
  4. A proposed commit message in the repo's style: a Conventional Commits subject
     and the `Co-Authored-By: Claude Opus 4.8 (1M context)
     <noreply@anthropic.com>` trailer from `CLAUDE.md`.
- Wait. The owner reviews the staged diff, makes manual edits, or requests changes.
  Commit only if the owner explicitly says to.

## Do not

- Do not extend `branded-id`; `ty-math` is the crate that grows here.
- Do not mix vmax and non-vmax edits in one commit.
- Do not change any wire format, golden, or the `voxcore` model; every edit is
  consumer-side arithmetic.
- Do not build a larger Track C primitive before filing it as an item.
- Do not commit, push, or amend without explicit approval.
