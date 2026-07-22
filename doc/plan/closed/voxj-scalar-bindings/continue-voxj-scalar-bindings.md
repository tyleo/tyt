# Continue the voxj scalar-bindings work

You are resuming a multi-session effort adding scalar bindings to the voxj
format and migrating the workspace to them. The plan is written and its
design decisions are settled; only the README's open questions are still
live, and only the owner closes those. Your job each session is to advance
the plan by one reviewable chunk and then stop for review.

## Orient first, every session

1. Read `doc/plan/open/voxj-scalar-bindings/README.md` for the format delta,
   the settled decisions (fixed; do not reopen), and the open questions.
2. Read `doc/plan/open/voxj-scalar-bindings/checklist.md`, the phased task
   list, and
   `doc/plan/open/voxj-scalar-bindings/reference/implementation-decisions.md`,
   the log of code-level choices made so far.
3. Run `git log --oneline -15` and `git status`. This repo commits directly
   on `main`; do not branch.
4. In phases 1 and 2 the working text is
   `doc/plan/open/voxj-scalar-bindings/reference/format-design.md`. Once the
   phase 2 spec commit lands, the authoritative format doc is
   `projects/voxel-codecs/voxj/docs/voxel-json-file-format.md`; use it for
   any detail the plan leaves implicit.

## Pick the work

- The next work is the first checklist item still unchecked, taken in order.
  Phases are in strict dependency order; do not skip ahead. The spec does not
  move before the design is approved, and no code moves before the spec
  lands.
- Phase 1 items are owner conversations: present the relevant
  `format-design.md` text and the open question, incorporate the answer, and
  fold it into the draft and the README.
- Phases 3 to 7 are deliberately coarse. Refine the current phase into
  commit-sized items when you reach it, then take the smallest coherent
  chunk that compiles and whose tests pass on its own. One commit-sized,
  reviewable change per session.
- Before starting, state in one line which phase and chunk you are doing.

## Do the work

- Follow `CLAUDE.md`: Rust edition 2024, consolidated nested `use`
  statements, one public type per file named in snake_case for the type, doc
  comments on public items, the module and feature-gate conventions, and
  ASCII-only comments and docs (no em dashes or ellipses), wrapped near 80
  columns.
- Regenerate the fixtures your chunk breaks in the same change; every
  fixture is inline in tests.
- Record any non-obvious code-level decision in
  `reference/implementation-decisions.md` (final binding type names, voxcore
  scalar-binding storage, converter scope, and so on).
- Check off the checklist items you complete, changing `[ ]` to `[x]`.
- Verify before staging: `cargo check`, `cargo fmt --all`, `cargo clippy
  --workspace --all-targets -- -D warnings`, then `cargo test -p` the crates
  you touched. The workspace will be red between crate phases; if a
  downstream crate cannot compile yet because its upstream just changed, say
  so plainly and scope your checks to the crates in play.

## Stage, do not commit

- `git add` everything you changed, including checklist and decision-log
  edits. Do not `git commit`, `git push`, or amend.
- Then stop and present for review:
  1. A short summary of what changed and why.
  2. The files touched, grouped logically.
  3. Test and lint results, or what could not run yet and why.
  4. A proposed commit message in the repo's style: a Conventional Commits
     subject and the `Co-Authored-By` trailer `CLAUDE.md` and the harness
     specify.
- Wait. The user reviews the staged diff, edits, or requests changes; treat
  any mid-session comment as an adjustment to the current chunk. Commit only
  if the user explicitly tells you to.

## Do not

- Do not reopen the settled decisions; the open questions close only with
  the owner, in phase 1.
- Do not start the spec rewrite before `format-design.md` is approved, or
  code before the spec lands.
- Do not run more than one reviewable chunk in a session.
- Do not commit, push, or amend without explicit approval.
