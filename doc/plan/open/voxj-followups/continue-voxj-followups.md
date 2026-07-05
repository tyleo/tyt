# Continue the voxj follow-up capabilities

You are advancing one of three independent follow-up tracks enabled by the
redesigned voxj format. The predecessor port is closed and this work is additive.
Unlike that port, the design decisions here are OPEN, so a session either resolves
a track's decision with the owner or executes a track whose decision is already
resolved. One reviewable, staged chunk per session, then stop for review.

## Orient first, every session

1. Read `doc/plan/open/voxj-followups/README.md` for the three tracks, their
   blast radius, and the four open decisions.
2. Read `doc/plan/open/voxj-followups/checklist.md`, the per-track task list, and
   `doc/plan/open/voxj-followups/reference/implementation-decisions.md`, the log
   of code-level choices made so far.
3. Run `git log --oneline -15` and `git status`. Each track is its own branch off
   `main`; branch if you are on `main` and starting a track.
4. The authoritative format spec is
   `projects/voxel-codecs/voxj/docs/voxel-json-file-format.md`. It does not change
   here; use it for any color-space or encoding detail.

## Pick the work

- If the track you intend to work on still has an unresolved decision (Q1, Q2, or
  Q3), your job this session is to resolve it: present the framing and
  recommendation, take the owner's call, and write it into the README as a
  **Decision** line the way the closed redesign plan records its decisions. Do not
  start coding a track on an unresolved decision.
- If the decision is resolved, take the first unchecked `[ ]` item in that track's
  phase. The three tracks are independent; do them in any order, but do not
  interleave two tracks in one session.
- State in one line which track and item you are doing.

## Do the work

- Follow `CLAUDE.md`: Rust edition 2024, consolidated nested `use`, one public
  type per file named in snake_case, doc comments on public items, the module and
  feature-gate conventions, comments wrapped to 80 columns, and only ASCII in
  comments.
- Rebuild only the touched crate's fixtures; the wire format does not change.
- Record any non-obvious code-level decision in
  `reference/implementation-decisions.md`.
- Check off the checklist items you complete, changing `[ ]` to `[x]`.
- Verify before staging: `cargo check`, then `cargo fmt --all`, then
  `cargo clippy --workspace --all-targets -- -D warnings`, then `cargo test -p`
  the crate or crates you touched. The workspace stays green throughout, since
  every track is additive.

## Stage, do not commit

- `git add` everything you changed, including the checklist and decision-log
  edits. Do not `git commit`, `git push`, or amend.
- Then stop and present for review:
  1. A short summary of what changed and why.
  2. The files touched, grouped logically.
  3. Test and lint results.
  4. A proposed commit message in the repo's style: a Conventional Commits
     subject and the `Co-Authored-By: Claude Opus 4.8 (1M context)
     <noreply@anthropic.com>` trailer from `CLAUDE.md`.
- Wait. The owner will review the staged diff, make manual edits, or request
  changes. Commit only if the owner explicitly tells you to.

## Do not

- Do not code a track whose decision is still open; resolve it first.
- Do not run more than one track, or the whole plan, in one session.
- Do not touch the wire format or the format spec; every track is consumer-side.
- Do not commit, push, or amend without explicit approval.
