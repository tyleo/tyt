# Continue the naming sweep

You are advancing a per-crate naming sweep across `voxcore`, `voxsmith`, and
`vxl`: bindings holding branded ids end in `_id` (`_ids` for collections),
entity-named index bindings end in `_index`, and value pool is spelled out
wherever a bare "pool" means one. The `voxj` and `voxj-codec` crates are the
reference for the target style. One crate per iteration, in order.

## Orient first, every session

1. Read `doc/plan/open/naming-sweep/README.md`: the three rules, the
   id-pool guard, and the decided Q1, Q2, and Q3.
2. Read `doc/plan/open/naming-sweep/checklist.md`,
   `doc/plan/open/naming-sweep/reference/survey.md`, and
   `doc/plan/open/naming-sweep/reference/ambiguous-mentions.md` if it
   exists.
3. Run `git log --oneline -10` and `git status`. Work on `main` unless the
   owner says otherwise.

## The one rule that must not be botched

`voxcore` has two "pool" entities. A value pool holds property values
(`VoxValuePool`, anything typed `U32Id<BVoxValuePool>`, a binding fetched
from the value pools). A branded-id pool is the id allocator behind an
entity listing (palette, object, node, layer, voxel, property, material,
and the value ids inside a value pool). Only value-pool mentions are
renamed. When a sentence could read either way, resolve it from the
surrounding code, not the wording, and log the call in
`reference/ambiguous-mentions.md` (file, line, ruling, why). `voxsmith` and
`vxl` have no id-pool mentions, so the rename is a safe blanket there.

## Do the work

- Take the first iteration with unchecked items and advance it item by
  item. Do not start the next crate's iteration until the current one's
  gate passes.
- The survey's line numbers are from 2026-07-29 and drift; re-grep before
  editing.
- Follow `CLAUDE.md`: consolidated nested `use`, one public item per file
  named for it, doc comments on public items, comments wrapped to 80
  columns, ASCII only.
- Renamed `format!` placeholders keep the rendered text byte-identical;
  message text edits happen only in the iteration's dedicated message
  commit.
- Check items off as they land. At the iteration's end run its gate:
  `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D
  warnings`, `cargo test -p voxcore -p voxsmith -p vxl`, and the gate greps
  from the checklist's ground rules.

## Stage, do not commit

- `git add` everything, including checklist and reference edits. Do not
  commit, push, or amend.
- Present for review: what changed and why, the files grouped logically,
  lint and test results, and a proposed commit message in the repo's style
  (a Conventional Commits subject, `refactor(voxel)!` for a breaking
  voxcore rename, plus the assistant Co-Authored-By trailer from
  `CLAUDE.md`).
- Wait for the owner. Commit only on explicit approval.

## Do not

- Do not rename a branded-id pool mention to `value_pool`; its prose
  expands to "id pool" forms only, in iteration 1's final docs commit.
- Do not touch the format spec, the voxj wire keys, or the Voxel-Max
  wire-struct field names.
- Do not rename an accessor's bare `id` subject parameter; every other
  bare-`id` binding takes its entity name (Q2).
- Do not commit, push, or amend without explicit approval.
