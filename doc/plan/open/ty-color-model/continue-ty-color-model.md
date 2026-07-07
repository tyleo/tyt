# Continue the palette-style color model migration

You are resuming a multi-session migration of `ty-math`'s color types onto the
`palette` crate's model: the color space is the type identity, the component type
is a separate generic parameter. This is execution, not planning. The design study
weighed four options and is closed; the owner chose the full generic. Your job each
session is to advance the migration by one reviewable, staged chunk and then stop
for the owner to review.

## Orient first, every session

1. Read `doc/plan/open/ty-color-model/README.md` for the model, the closed decision
   (`TySrgba<T>` replaces both sRGB storages, `TyLinSrgba<T>` replaces the linear
   type, non-color channels leave the color namespace), the four friction points
   (`Eq`/`Hash`/`Ord`, component conversion, the fbx serde, the HSV family), and the
   blast radius. Treat the decision as fixed; do not reopen the four-option study.
2. Read `doc/plan/open/ty-color-model/checklist.md` for the nine steps (S1-S9) across
   three phases and the carried-over items from the closed ty-math adoption plan.
3. Read `doc/plan/open/ty-color-model/reference/color-model-study.md` for the census
   and prior art (`palette`, `bevy_color`, `egui`, `kolor`) when the plan leaves a
   detail implicit.
4. Log every non-obvious code-level decision in
   `doc/plan/open/ty-color-model/reference/implementation-decisions.md` (a confirmed
   method name, how `into_format` quantizes, the HSV verdict, and so on). If it
   already exists, read it first and stay consistent with the choices recorded there;
   if not, create it the first time you make a call worth recording.
5. Run `git log --oneline -15` and `git status`. Confirm the checklist line numbers
   still hold at the keyboard before editing; they are from the design-study census.

## Pick the work

- The next work is the first checklist step with an unchecked `[ ]`, taken in order.
  The three phases are strict dependency order: Phase 1 lands the new types beside
  the old (additive), Phase 2 migrates consumers, Phase 3 does the serde rename and
  removal. The workspace stays green at every step, so do not skip ahead.
- If a step is large, for example S5 (~17 `TySrgbaColor` sites), split it into the
  smallest coherent chunk that compiles and whose tests pass on its own, and do only
  that chunk this session. One commit-sized, reviewable change per session.
- State in one line which step and chunk you are doing.

## Do the work

- Follow `CLAUDE.md`: Rust edition 2024, consolidated nested `use` statements, one
  public type per file named in snake_case for the type, doc comments on public
  items, comments wrapped to 80 columns, and only ASCII (no em dashes, ellipses, or
  other non-keyboard characters).
- Additive first. S1-S3 add `TySrgba<T>` and `TyLinSrgba<T>` beside the existing
  types; the old types are removed only at S8. Every staged chunk keeps the whole
  workspace building; if a chunk cannot compile on its own, it is scoped wrong.
- No wire format changes. The one deliberate serde touch is S7 (rename the fbx
  per-point-color serde to `TySrgbaSerde`, keeping the `r`/`g`/`b`/`a` keys), and
  even that stays byte-identical. If any golden or round-trip output changes, stop:
  this migration is type-only, not behavior-changing.
- `voxcore` is untouched; its `VoxValuePool` arrays and named variants stay as-is,
  and the sRGB-float concept lives there, not in a `ty-math` color type.
- Check off the checklist items you complete, changing `[ ]` to `[x]`.
- Verify before staging: `cargo check`, then `cargo fmt --all`, then `cargo clippy
  --workspace --all-targets -- -D warnings`, then `cargo test -p` the crates in play
  (`ty-math` for S1-S4; `voxsmith` and `vxl` for S5-S6; `tyt-fbx` and `tyt-injection`
  for S7; `--workspace` for S8-S9).

## Stage, do not commit

- `git add` everything you changed, including the checklist and the decision-log
  edits. Do not `git commit`, `git push`, or amend.
- Then stop and present for review:
  1. A short summary of what changed and why.
  2. The files touched, grouped logically.
  3. Test and lint results, or what could not run yet and why.
  4. A proposed commit message in the repo's style: a Conventional Commits subject
     and the `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`
     trailer from `CLAUDE.md`.
- Wait. The owner reviews the staged diff, makes manual edits, or requests changes.
  Treat any mid-session comment as an adjustment to the current chunk. Commit only if
  the owner explicitly says to.

## Do not

- Do not reopen the closed decision or re-run the four-option study; the full generic
  `TySrgba<T>` is chosen.
- Do not change any wire format except the deliberately-gated fbx point-color serde
  in S7, and keep even that byte-identical.
- Do not touch `voxcore`; its `VoxValuePool` model and serde stay as-is.
- Do not remove the old types before S8, skip a phase, or run the whole plan in one
  session.
- Do not half-remove the HSV family; S4 decides it one way or the other.
- Do not commit, push, or amend without explicit approval.
