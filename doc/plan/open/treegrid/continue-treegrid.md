# Continue the treegrid build

You are executing a multi-session plan that extracts one shared
hierarchical-data renderer -- the `treegrid` crate at
`projects/utilities/treegrid` -- and migrates the read commands onto it:
`vxl palette show`, `vxl hierarchy show`, `vxl palette list`,
`tyt vmax hierarchy`, `tyt fbx hierarchy`, and the `vxl info` /
`validate` / `list` tables, plus a small `pathspec` coda. This is
execution, not planning: the design is owner-reviewed and its eleven
decisions are closed. Your job each session is to advance the plan by
one reviewable, staged chunk and then stop for the user to review. If
no checklist item is checked yet, the first session starts at S1.

## Orient first, every session

1. Read `doc/plan/open/treegrid/README.md` for the model (populate a
   branded-id forest, then render), the six layouts, the three label
   modes, the boundaries (selection, sampling policy, IO, and clap stay
   in the commands), and the eleven decisions. Treat the decisions as
   fixed; do not reopen them.
2. Read `doc/plan/open/treegrid/checklist.md`, the phased task list
   (S1-S19 across seven phases), and
   `doc/plan/open/treegrid/reference/rendering-spec.md`, the exact
   contract for every layout, label mode, and the cell format matrix.
   Any behavior question is settled by the spec; read
   `doc/plan/open/treegrid/reference/design-notes.md` when you need the
   rationale behind a rule.
3. Read `doc/plan/open/treegrid/reference/implementation-decisions.md`,
   the log of code-level choices made so far; stay consistent with it
   and append as you go.
4. Run `git log --oneline -15` and `git status`. Stay on the branch you
   find checked out; do not create or switch branches unless the user
   says to.
5. The code being replaced is the reference for parity: `vxl`'s
   `implementation/palette_show.rs`, `hierarchy_show.rs`, and
   `palette_list.rs`; `tyt-vmax/src/commands/hierarchy.rs`;
   `tyt-fbx/src/commands/hierarchy.rs`; and the arena pattern in
   `projects/utilities/voxcore/src/vox_hierarchy_node.rs`.

## Pick the work

- The next work is the first unchecked `[ ]` step, taken in order.
  Phases 1 through 4 and 6 are in dependency order; do not skip ahead
  within them. Two exceptions: phase 5 (`tyt fbx hierarchy`) is
  severable and may slip past phase 6, and phase 7 (the `pathspec`
  `TreeSelection` closure) is independent and may land at any point --
  pull S18 forward if phase 5 runs first, since S14 builds on it.
- If a step is large (S1, S7), split it into the smallest coherent
  chunk that compiles and whose tests pass on its own, and do only that
  chunk this session. One commit-sized, reviewable change per session.
- Before starting, state in one line which step and chunk you are doing.

## Do the work

- Follow `CLAUDE.md`: Rust edition 2024, consolidated nested `use`
  statements, one public type per file named in snake_case for the
  type, private `mod` declarations with `pub use` re-exports, doc
  comments on public items, comments wrapped to 80 columns, and only
  ASCII in comments and docs (no em dashes or ellipses).
- The crate's dependency is `branded-id`, plus two optional features:
  `json` gating `serde_json` (`preserve_order`) with the value JSON
  forms and the JSON layouts, and `ty-math` gating the typed-color
  constructors. No clap, libc, tyt-common, or
  tyt-injection; no `Dependencies` trait and no `impl` feature.
  Publishable metadata like the sibling utilities crates; add it to
  workspace `members` and `[patch.crates-io]`, but do not publish --
  that is a deferred item the owner triggers.
- Parity is the bar: phases 3 and 4 end with byte-identical default
  command output, and phase 2 keeps the default (`rows` + `concat`)
  output byte-identical -- its flag renames and JSON envelope change
  are deliberate and called out in the commit message.
- If the implementation must disagree with the rendering spec, fix one
  of them in the same commit and say which in
  `reference/implementation-decisions.md`.
- Some steps embed a decision (the number-pool gray-swatch rule at S7,
  `Bare` versus `Quoted` at S11 and S12): make the call, log it in
  implementation decisions, and move on.
- Check off the checklist items you complete, changing `[ ]` to `[x]`.
- Verify before staging: `cargo check`, then `cargo fmt --all`, then
  `cargo clippy --workspace --all-targets -- -D warnings`, then
  `cargo test -p` the crate or crates you touched (`treegrid` in phase
  1; `vxl` in phases 2, 3, and 6; `tyt-vmax` in phase 4; `tyt-fbx` in
  phase 5; `pathspec` plus both adopters in phase 7).

## Stage, do not commit

- `git add` everything you changed, including the checklist and
  decision-log edits. Do not `git commit`, `git push`, or amend.
- Then stop and present for review:
  1. A short summary of what changed and why.
  2. The files touched, grouped logically.
  3. Test and lint results.
  4. A proposed commit message in the repo's style: a Conventional
     Commits subject (`feat(treegrid)` for crate steps, `feat(vxl)!`
     for the breaking S7 adoption, and so on) and the
     `Co-Authored-By: Claude ... <noreply@anthropic.com>` trailer for
     the model doing the work, in the form recent commits use.
- Wait. The user reviews the staged diff, makes manual edits, or
  requests changes. Treat any mid-session comment as an adjustment to
  the current chunk. Commit only if the user explicitly says to.

## Do not

- Do not reopen the eleven closed decisions: no visitor trait, no
  re-parenting in the arena, no auto-detected table shape, no
  label-keyed JSON objects.
- Do not let selection, glob filtering, sampling or precision policy,
  terminal-width detection, or clap types into the crate; it renders
  what it is handed. The one shared selection piece, `TreeSelection`,
  lands in `pathspec` (phase 7), not in treegrid.
- Do not change default byte output in the parity phases; if a golden
  changes outside the deliberate phase 2 JSON change, stop and say so.
- Do not skip ahead in phases 1-4 and 6, and do not run multiple steps
  in one session.
- Do not publish to crates.io.
- Do not commit, push, or amend without explicit approval.
