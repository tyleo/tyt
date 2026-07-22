# Continue the palette adoption

You are resuming a migration that replaces ty-math's hand-rolled color types with
`type Ty... = palette::...` aliases and moves every consumer onto palette's own
methods. This is execution, not planning; the design is settled. Your job each
session is to advance the migration by one reviewable chunk and then stop for the
owner to review.

## Orient first, every session

1. Read `doc/plan/open/palette-adoption/README.md` for the goal, the narrow
   meaning of "palette doesn't leak" (no consumer names the `palette` crate; tyt
   names stay the vocabulary; field/method renames are accepted), the decisions,
   the six frictions, and the commit strategy (one clean commit, staged across
   sessions - the alias flip is atomic, there is no green intermediate).
2. Read `doc/plan/open/palette-adoption/checklist.md` for the nine steps across
   three phases.
3. Read `doc/plan/open/palette-adoption/reference/palette-api-map.md` for the
   verified alias lines, the method/operator map, trait/derive facts, and the
   friction resolutions. Read `reference/consumer-census.md` for the per-file
   sites and the foreign-`.r/.g/.b` traps to leave alone.
4. Log every non-obvious code-level decision in
   `reference/implementation-decisions.md` (the confirmed `into_format` rounding
   result, the exact `TyHexColor` shape, whether the treegrid `TyFloatExt` bound
   dropped, any Lab re-baseline). Create it the first time; read and stay
   consistent with it thereafter.
5. Run `git log --oneline -15` and `git status`. Confirm the checklist line
   numbers still hold at the keyboard before editing.

## Pick the work

- The next work is the first checklist step with an unchecked `[ ]`, in order.
  Phase 1 (S1-S2) lands the ty-math aliases + glue; Phase 2 (S3-S7) migrates
  consumers crate by crate; Phase 3 (S8-S9) sweeps green and commits. The
  workspace is RED from S2 until S8 by design - verify sub-parts with `cargo check
  -p <crate>`, not `--workspace`, mid-migration.
- If a step is large, split it into the smallest coherent chunk (e.g. one file or
  one converter within S3/S4) and do only that this session. State in one line
  which step and chunk you are doing.

## Do the work

- Follow `CLAUDE.md`: edition 2024, consolidated nested `use`, one public item per
  file (a `pub type` alias counts; extension traits are one trait per file named
  for the trait), doc comments on public items, 80-column ASCII comments.
- Prefer palette's behavior; do NOT chase byte-exactness. Re-baseline internal
  tests/goldens that legitimately shift and note them. The verified frictions:
  out-of-gamut sign-extension is dropped, CIELAB drifts a few sig figs (Oklab is
  identical), `into_format::<u8>()` may differ by +/-1 LSB on an exact `.5`.
- Do NOT let `palette::` appear in any consumer crate. If a consumer would need
  it, add the missing re-export to ty-math instead.
- Keep the serde DTO (`TySrgbaSerde`, `r/g/b/a`) and palette's `serializing`
  feature OFF; only the `From<TySrgba>` body reads palette accessors. The fbx
  JSON must stay byte-identical (a pinned test guards it).
- Watch the traps: `to_srgb` is DROP-ALPHA in some sites and TRANSFER in others;
  `componentwise_multiply` -> by-value `*`; `into_format` on `Srgba` needs TWO
  turbofish params; the Lab alias MUST pin `D65`; leave every foreign voxel/color
  `.r/.g/.b` and every `TyVector3` method alone.
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
  subject with the `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`
  trailer.
- Wait. Treat any mid-session comment as an adjustment to the current chunk.

## Do not

- Do not reopen the design (aliases, kept glue, dropped byte-exactness are
  settled) or add a palette-reduction/quantizer (out of scope; `reduce_palette`'s
  clustering stays voxsmith's).
- Do not enable palette's `serializing` / `named` / `random` features, or
  serialize a palette type; keep the DTO.
- Do not let `palette::` leak into a consumer crate.
- Do not touch `voxcore`, or remove the `ty_array_conversions!` macro (vectors
  still use it).
- Do not commit, push, or amend without explicit approval; land one clean commit
  at S9.
