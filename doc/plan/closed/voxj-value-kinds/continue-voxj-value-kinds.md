# Continue the voxj value-kinds work

You are advancing the voxj value-kinds change: the target spec text is
agreed with the owner, lands as one spec commit, and the code follows in
two moves, the `Factor` property rename first and then the value pool
kinds becoming pure JSON shapes (the six color kinds and the `min`/`max`
bounds go, six vector kinds arrive, every color is linear light), one
crate per iteration in dependency order.

## Orient first, every session

1. Read `doc/plan/open/voxj-value-kinds/README.md`: the design is settled;
   do not reopen it. New findings go to the survey and the decisions log,
   not the README.
2. Read `doc/plan/open/voxj-value-kinds/checklist.md`,
   `doc/plan/open/voxj-value-kinds/reference/survey.md` (line numbers are
   from 2026-08-01 and drift; re-grep before editing), and
   `doc/plan/open/voxj-value-kinds/reference/implementation-decisions.md`.
3. Run `git log --oneline -10` and `git status`. Work on `main` unless the
   owner says otherwise.
4. In iterations 1 and 2 the working text is
   `doc/plan/open/voxj-value-kinds/reference/format-design.md`. Once the
   iteration 2 spec commit lands, the authoritative format doc is
   `projects/voxel-codecs/voxj/docs/voxel-json-file-format.md`; use it for
   any detail the plan leaves implicit. Iterations 9 and 10 work the same
   way on `doc/plan/open/voxj-value-kinds/reference/palette-show-design.md`:
   the owner approves the palette show design before iteration 10 codes
   it, and `--type` deletes with that iteration.

## The rules that must not be botched

- Order is strict: the spec commit does not start before the owner
  approves format-design.md, and no code moves before the spec lands.
  The rename lands before the kind iterations.
- Iteration 1 and iteration 9 items are owner conversations: present
  the relevant design-page text, incorporate the answer, and fold it
  into the draft. Only the owner closes a design question.
- The glTF wire keeps its `Factor` field names: the serde_json output
  keys in `material_document.rs`, the `gltf` crate accessor calls and the
  test GLB builders in `from_gltf_bytes.rs`, and the spec table's per-row
  glTF citations. Only voxj property names rename. The
  `material_document.rs` tuple pairing the voxj constant with the wire
  literal stays two strings.
- The vmax `--color-format` (`png`/`plist`/`all`) is an unrelated flag;
  only the voxj one deletes.
- voxsmith and vxl spell property names through the `gltf_attributes`
  constants; voxj, voxj-codec, and voxcore keep plain literals.
- The asset `submodules/tyt-assets/scratch/energy-turret.voxj`
  regenerates once, at closeout, never earlier.
- The plan must not stop after iteration 3: the rename alone fails
  silently at a crossing, and the README accepts it only because the kind
  break lands in this same plan.

## Do the work

- Take the first iteration with unchecked items and advance it item by
  item. Do not start the next iteration until the current gate passes.
- Iterations 1 through 3 keep the workspace green. Iterations 4 through 7
  leave downstream crates red until iteration 8; scope clippy and tests
  to the crates in play and say so plainly, spelling out the crate's
  features (`-p voxj --features serde`, `-p voxsmith --all-features`) or
  the run compiles none of the work. The pre-commit hook's workspace
  clippy cannot pass inside that window; its header documents the bypass.
- Follow `CLAUDE.md`: consolidated nested `use`, one public item per file
  named for it, doc comments on public items, comments wrapped to a
  filled 80 columns, ASCII only, errors over silent fallbacks.
- Fixtures are inline; regenerate them in the same commit that breaks
  them.
- Record non-obvious choices in
  `reference/implementation-decisions.md`: the sentinel serde module's
  name and surface, the vocabulary range check (home, signature, and
  whether it absorbs `scalar_range`), what value flaw checking voxcore
  keeps, the `mesh_material.rs` `emissive_factor` ruling.
- Check items off as they land.

## Commit per concern, review per iteration

- Drive the current iteration to its gate in one session where it fits
  (iteration 7 may take more than one; say plainly where you stopped),
  committing each concern as it lands: a Conventional Commits subject
  (`docs(voxj)` for spec commits, `docs(plan)` for plan-page commits,
  `feat`/`refactor` with `!` where a public surface breaks) plus the
  Co-Authored-By trailer from the harness. Include checklist and
  reference edits in the concern's commit.
- At the gate, present the iteration for review: the commits, what
  changed and why, lint and test results (and which crates could not run
  inside the red window), and the gate grep output.
- Do not push or amend. The owner reviews the landed series and says how
  to proceed.

## Do not

- Do not reopen the README's decisions or edit its content.
- Do not start the spec commit before format-design.md is approved, or
  code before the spec lands.
- Do not touch closed plan pages; the open vxl-commands pages sweep at
  closeout.
- Do not rename a glTF wire field or the vmax `--color-format`.
- Do not regenerate the asset before closeout.
- Do not push or amend without explicit approval.
