# Ambiguous pool mentions

Rulings on bare "pool" mentions whose entity took real judgment, per the
[checklist](../checklist.md) ground rules. One line each: file, line (at
ruling time), ruling, why.

- `vox_palette.rs:280` — value pool. "Value ids point into the referenced
  pools": value ids live in value pools; the survey's id-pool list had
  this line, overruled from context.
- `vox_main.rs:1211` — id pool. "removals have left the pools with
  holes": the retention checks concern the layer, palette, and material
  id pools, not value pools.
- `vox_main.rs:970` — id pool. "Compact each palette's own pools":
  `VoxPalette::gc` compacts the palette's property and material id
  pools.
- `vox_main.rs:972` — id pool. "the palette pool's whole id space": the
  palette id pool's id space.
- `vox_main.rs:966` — id pool. "in sync with the pre-gc pool" beside the
  value-pool column: the column pairs with the value-pool id pool. Note:
  the line also says "value-pool", so the gate grep's line-based
  exclusion hides it; the Q1 pass must fix it by hand.
- `vox_value_pool.rs:283` — id pool. "Rebuild it against the cloned
  pool": the column pairs with the value-id id pool the previous
  sentence names.
- `vox_value_pool.rs:22` — both entities in one line. "Value id pool.
  Its listing order is the pool's value order": the first mention is the
  id pool of value ids and stays; the second is the value pool and was
  spelled out.
- `vox_main.rs:2104` — already correct, wrap artifact. "Value\npools"
  split across comment lines read as a bare-pool line to the line-based
  gate grep; rewrapped so "Value pools" sits on one line, wording
  unchanged.
- `vox_gc_remap.rs:12` — value pool. "indexed by the pool's old id" on
  the `value_pool_values` field: the index is the value pool's pre-gc
  id. Hidden from the docs-commit grep by "value pool" earlier on the
  line; landed with the id-pool commit.
- `vox_palette.rs:138` — value pool. "a value in the pool that property
  draws from": the wrap put "a" on the previous line, so the
  docs-commit pattern missed it; landed with the id-pool commit.
- `vox_main.rs:993` — id pool. "compact its own reference pool": the
  object's layer references live in its layer id pool, so it reads
  "its own layer id pool".
- `vox_value_pool.rs:11` — both entities in one line. "A shared pool of
  values ... keyed by a pool of value ids": the first defines the value
  pool itself and reads "A shared value pool"; the second is the id
  pool of value ids.
- `palette_show.rs:216` — already correct, wrap artifact. The docs commit's
  rewrap split "value pool" across two lines, which the line-based gate
  grep reads as a bare mention; rewrapped so it sits on one line, wording
  unchanged.
- `vox_main.rs:433`, `471`, and `896` — value pool. Each line already said
  "value pool" or "Value pools" earlier, so the gate grep's line-based
  exclusion hid the trailing bare mention through iteration 1; spelled out
  during iteration 3's gate.
