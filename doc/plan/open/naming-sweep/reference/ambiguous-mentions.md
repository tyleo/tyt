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
