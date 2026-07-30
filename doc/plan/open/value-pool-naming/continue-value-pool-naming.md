# Continue the value-pool naming sweep

You are advancing a naming sweep: spell "value pool" out across `voxcore`,
`voxsmith`, and `vxl` wherever a bare "pool" means a value pool. The `voxj`
and `voxj-codec` crates are already swept and are the reference for the target
style. One phase per session, then stop for review.

## Orient first, every session

1. Read `doc/plan/open/value-pool-naming/README.md`: the rule, the two pool
   entities, and Q1.
2. Read `doc/plan/open/value-pool-naming/checklist.md` and
   `doc/plan/open/value-pool-naming/reference/ambiguous-mentions.md` if it
   exists.
3. Run `git log --oneline -10` and `git status`. Work on `main` unless the
   owner says otherwise.

## The one rule that must not be botched

The workspace has two "pool" entities. A value pool holds property values
(`VoxValuePool`, anything typed `U32Id<BVoxValuePool>`, a binding fetched from
`value_pools`). A branded-id pool is the id allocator behind an entity listing
(palette, object, node, layer, voxel, property, material, and the value ids
inside a value pool). Only value-pool mentions are renamed. When a sentence
could read either way, resolve it from the surrounding code, not the wording,
and log the call in `reference/ambiguous-mentions.md` (file, line, ruling,
why).

## Do the work

- Take the first phase with unchecked items; do not start the next phase in
  the same session.
- Follow `CLAUDE.md`: consolidated nested `use`, one public item per file,
  doc comments on public items, comments wrapped to 80 columns, ASCII only.
- Renamed `format!` placeholders keep the surrounding literal text
  byte-identical; message-text edits happen only in Phase D's dedicated
  message commit.
- Check items off as they land. After the phase, run its gate:
  `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test -p voxcore -p voxsmith -p vxl`, and the gate grep from the
  checklist's ground rules.

## Stage, do not commit

- `git add` everything, including checklist and reference edits. Do not
  commit, push, or amend.
- Present for review: what changed and why, the files grouped logically, lint
  and test results, and a proposed commit message in the repo's style (a
  Conventional Commits subject, `refactor(voxel)!` for Phase A, plus the
  assistant's Co-Authored-By trailer).
- Wait for the owner. Commit only on explicit approval.

## Do not

- Do not rename a branded-id pool mention to `value_pool`.
- Do not touch `voxj`, `voxj-codec`, the format spec, or any JSON wire name.
- Do not run more than one phase per session.
- Do not start Phase E unless Q1 is resolved to A.
