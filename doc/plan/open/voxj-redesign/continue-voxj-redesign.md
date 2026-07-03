# Continue the voxj redesign port

You are resuming a multi-session port of this repo to the redesigned voxj format.
This is execution, not planning: the plan is already written and its eight design
decisions are closed. Your job each session is to advance it by one reviewable,
staged chunk and then stop for the user to review.

## Orient first, every session

1. Read `doc/plan/open/voxj-redesign/README.md` for the four format deltas, the
   crate blast radius, and the eight closed decisions. Treat those decisions as
   fixed; do not reopen them.
2. Read `doc/plan/open/voxj-redesign/checklist.md`, the phased task list, and
   `doc/plan/open/voxj-redesign/reference/implementation-decisions.md`, the log of
   code-level choices made so far.
3. Run `git log --oneline -15` and `git status`. Confirm you are on the
   `voxj-redesign` branch and stay on it; do not branch.
4. The authoritative format spec is
   `projects/voxel-codecs/voxj/docs/voxel-json-file-format.md`. Use it for any
   detail the plan leaves implicit.

## Pick the work

- The next work is the first checklist phase that still has unchecked `[ ]`
  items, taken in order. Phases are in strict dependency order; do not skip ahead.
- If that phase is large, for example Phase 3 `voxcore`, split it into the
  smallest coherent chunk that compiles and whose tests pass on its own, and do
  only that chunk this session. One commit-sized, reviewable change per session.
- Before starting, state in one line which phase and chunk you are doing.

## Do the work

- Follow `CLAUDE.md`: Rust edition 2024, consolidated nested `use` statements, one
  public type per file named in snake_case for the type, doc comments on public
  items, the module and feature-gate conventions, comments wrapped to 80 columns,
  and only ASCII in comments (no em dashes, ellipses, or other non-keyboard
  characters).
- Rebuild the affected crate's test fixtures in the same change; every existing
  fixture encodes the old palette shape.
- Record any non-obvious code-level decision in
  `reference/implementation-decisions.md` (how min/max is modeled in serde, how
  voxcore stores color, how the vmax fold reconstructs indices, and so on).
- Check off the checklist items you complete, changing `[ ]` to `[x]`.
- Verify before staging: `cargo check`, then `cargo fmt --all`, then
  `cargo clippy --workspace --all-targets -- -D warnings`, then `cargo test -p`
  the crate or crates you touched. The workspace will not fully build mid-port; if
  a downstream crate cannot compile yet because its upstream just changed, say so
  plainly and scope your checks to the crates in play.

## Stage, do not commit

- `git add` everything you changed, including the checklist and decision-log
  edits. Do not `git commit`, `git push`, or amend.
- Then stop and present for review:
  1. A short summary of what changed and why.
  2. The files touched, grouped logically.
  3. Test and lint results, or what could not run yet and why.
  4. A proposed commit message in the repo's style: a Conventional Commits
     subject and the `Co-Authored-By: Claude Opus 4.8 (1M context)
     <noreply@anthropic.com>` trailer from `CLAUDE.md`.
- Wait. The user will review the staged diff, make manual edits, or request
  changes to the staged files. Treat any mid-session comment as an adjustment to
  the current chunk. Commit only if the user explicitly tells you to.

## Do not

- Do not re-litigate the eight closed decisions.
- Do not run the whole port in one session; one reviewable chunk, then stop.
- Do not commit, push, or amend without explicit approval.
