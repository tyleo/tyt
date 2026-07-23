# Continue the glam adoption

You are resuming a migration that replaces ty-math's hand-rolled math types
(`TyVector2/3/4`, `TyQuaternion`, `TyMatrix4x4`, `TyPose`/`TyTransform`/
`TyUniformTrs`/`TyBounds`) with concrete `glam` aliases + a small set of extension
traits and moves every consumer onto glam's own methods. This is execution, not
planning; the design is settled. Each session, advance the migration by one
reviewable chunk and then stop for the owner to review.

## Orient first, every session

1. Read `doc/plan/open/glam-adoption/README.md` for the goal, the narrow meaning
   of "glam doesn't leak" (no consumer names `glam`; the `Ty...` names stay the
   vocabulary; method renames are accepted and preferred), the decisions, the
   frictions, and the commit strategy (one clean commit, staged across sessions -
   the alias flip is atomic, there is no green intermediate).
2. Read `doc/plan/open/glam-adoption/checklist.md` for the nine steps across three
   phases.
3. Read `doc/plan/open/glam-adoption/reference/glam-api-map.md` for the concrete
   alias table and the per-method DIRECT/EXT/DROP mapping. Read
   `reference/glam-facts.md` for the crate facts, the single-dependency wiring,
   the `debug-glam-assert` decision, and why glamx was evaluated and not adopted.
   Read `reference/consumer-census.md` for the per-file sites and the foreign
   look-alikes to leave alone.
4. Log every non-obvious code-level decision in
   `reference/implementation-decisions.md` (the resolved glam version, the exact
   ext-trait shapes, which trivia were dropped, the f32-round verification, the
   q->matrix->q and euler round-trip results, whether `from_rotation_matrix` kept
   `None` or asserts, any Debug/euler re-baseline). Create it the first time; read
   and stay consistent thereafter.
5. Run `git log --oneline -15` and `git status`. Confirm the checklist line
   numbers still hold at the keyboard before editing.

## Pick the work

- The next work is the first checklist step with an unchecked `[ ]`, in order.
  Phase 1 (S1-S2) lands the ty-math aliases + ext traits + composites; Phase 2
  (S3-S7) migrates consumers crate by crate; Phase 3 (S8-S9) sweeps green and
  commits. The workspace is RED from S2 until S8 by design - verify sub-parts with
  `cargo check -p <crate>`, not `--workspace`, mid-migration.
- S3 and S4 (voxsmith) are large. Split into the smallest coherent chunk (one
  converter or one internal file) and do only that this session. State in one line
  which step and chunk you are doing.

## Do the work

- Follow CLAUDE.md: edition 2024, consolidated nested `use`, one public item per
  file, ONE extension trait per file named for the trait, doc comments, 80-col
  ASCII comments, no em dashes. Composites get an `IDENTITY` const (glam naming),
  not an `identity()` fn.
- Prefer glam's behavior AND its names; do NOT chase byte-exactness. Re-baseline
  internal tests/goldens that legitimately shift and note them.
- **Fail-fast: use glam's strict methods directly** (`from_axis_angle`, `inverse`,
  `normalize`, `is_normalized`, `slerp`). Do NOT re-add the old ty defensive
  normalize/guard as a silent override. Normalize explicitly at the call site
  where an input is not provably unit; `debug-glam-assert` panics in tests point
  you at the misses. Only if an auto-fixing pattern genuinely recurs, add a
  DISTINCTLY NAMED variant (`from_axis_angle_normalized`), never a silent one.
- **Lift glam's algorithms for the residue:** `from_basis_vectors`/
  `from_rotation_matrix` delegate to `Quat::from_rotation_axes` (drop ty's
  trace-branch); `rotate_extents_abs` = `DMat3::from_quat(q)` with per-column
  `.abs()` then `* extents`.
- Do NOT let `glam::` appear in any consumer crate. If a consumer would need it,
  add the missing re-export (an alias or an ext method) to ty-math.
- Do NOT enable glam's `serde` or `glam-assert` (all-builds). Keep the serde DTO
  (`TyVector3Serde {x,y,z}`); the `{x,y,z}` JSON wire must stay byte-identical (a
  pinned test guards it).
- Watch the traps (census "Traps"): `dot(&o)`/`cross(&o)` become by-value;
  `componentwise_multiply` -> `*`; `component(i)` -> `[i]`; `magnitude` ->
  `length`; quaternion `new` -> `from_xyzw` and `identity()` -> `IDENTITY`;
  `from_column_arrays` needs a leading `&`; `scalar * q` -> `q * scalar`; leave
  every foreign `[f64;N]`/`voxel.*`/codec array and the value-source methods
  (`object.bounds()`, `node.transform.*`) whose return type is ty-math but whose
  line has no `Ty*` token.
- Check off the checklist items you complete, changing `[ ]` to `[x]`.
- Verify before staging: `cargo check` (the crates in play), `cargo fmt --all`,
  `cargo clippy --workspace --all-targets -- -D warnings` once the workspace is
  green again, and `cargo test -p` the crates you touched.

## Stage, do not commit

- `git add` everything you changed, including the checklist and decision-log
  edits. Do NOT `git commit`, `git push`, or amend - the migration lands as ONE
  clean commit at S9 on explicit owner approval.
- Then stop and present: a short summary of the chunk, the files touched grouped
  logically, test/lint results (or what could not run yet because the workspace
  is mid-flip and why), and - at S9 only - a proposed Conventional Commits
  subject with the `Co-Authored-By: Claude Opus 4.8 (1M context)
  <noreply@anthropic.com>` trailer.
- Wait. Treat any mid-session comment as an adjustment to the current chunk.

## Do not

- Do not reopen the design (concrete aliases, glam-only, fail-fast with
  debug-glam-assert, glam's names, lifted algorithms, hand-rolled composites,
  dropped byte-exactness are settled).
- Do not offer auto-normalizing/auto-fixing variants preemptively (the strict glam
  methods are the default and can be faster in the extreme); if one proves needed,
  name it distinctly, never as a silent override.
- Do not enable glam's serde or `glam-assert` (all-builds), or serialize a glam
  type; keep the DTO. Do not adopt glamx as a dependency.
- Do not let `glam::` leak into a consumer crate.
- Do not touch `voxcore`'s `VoxValuePool` or any voxel-codec wire format, and do
  not build features around glam's extras (`try_inverse`, `Vec3A`, camera).
- Do not commit, push, or amend without explicit approval; land one clean commit
  at S9.
