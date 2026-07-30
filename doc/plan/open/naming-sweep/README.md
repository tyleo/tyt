# Naming Sweep Plan

Status: **open.** Iterations 1 (`voxcore`) and 2 (`voxsmith`) are complete
and awaiting the owner's gate review; iteration 3 (`vxl`) is next.

This plan carries the naming rules the recent `voxcore` and voxj commit
series established across the crates that still miss them. Three rules, one
iteration per crate, and each iteration lands every rule for its crate
(identifiers, comments, and messages together) before the next crate starts.
The executable steps live in [checklist.md](checklist.md), the line-level
survey inventory in [reference/survey.md](reference/survey.md), and the
per-session resume prompt in
[continue-naming-sweep.md](continue-naming-sweep.md).

The reference commits: `7211714` suffixed voxcore's id-holding bindings with
`_id` and `_ids`, `b82b436` suffixed voxj-codec's index-holding bindings with
`_index`, and `375873f`, `13e4feb`, `c263460`, `f745331`, and `4c85783`
spelled value pool out across voxcore's types and the voxj crates. This plan
finishes that work in `voxcore` and extends it to `voxsmith` and `vxl`, one
crate at a time rather than one concern at a time.

## The three rules

- **Ids.** A binding holding a branded `U32Id<B*>` id (a field, parameter,
  local, loop binding, or closure argument) is named for its entity plus
  `_id`, `_ids` for a collection. A binding holding the entity value itself
  keeps the entity name. `vxl`'s treegrid node ids (`U32Id<BTreeGridNode>`)
  are branded ids and follow the rule.
- **Indices.** A binding holding a positional index and named for an entity
  carries `_index` (`palette_index`, `layer_index`, `node_index`). A bare `i`
  in a trivial enumerate is tolerated. The reverse error is not: `index` must
  never name a value that is actually a branded id.
- **Value pools.** A bare "pool" meaning a value pool becomes `value_pool` in
  identifiers and "value pool" in prose and messages. Spec kinds stay bare
  inside kind-qualified prose: "an `int` value pool", never "an `int-value`
  pool".

The one guard: `voxcore` also holds branded-id pools, the id allocators
behind the entity listings (palette, object, node, layer, voxel, property,
material, and the value ids inside a value pool). Those are a different
entity and keep their names. The survey found roughly 50 such mentions, all
in `voxcore` (`vox_object.rs`, `vox_runtime_state.rs`, `vox_gc_remap.rs`, the
gc section of `vox_main.rs`, and single lines elsewhere), and none in
`voxsmith` or `vxl`: in those two crates every "pool" is a value pool, so the
rename is a safe blanket there.

## What the survey found (2026-07-29)

**voxcore.** The id sweep landed almost completely; the residue is seven loop
bindings named bare `id` where every sibling loop writes `*_id`, and
`first_cycle_position` in `vox_main.rs`, whose `start` and `node` bindings
are node indices (its voxj-codec twin `check_acyclic.rs` was swept). The
value pool work is the larger half: three breaking API items remain
(`VoxProperty::pool_id`, the `pool_id` field on six `Error` variants, and
`VoxValuePool::clone_pool`, which the type-rename series missed), five
public parameter names with about ten rustdoc references, roughly 95
internal and 180 test bare-pool identifier occurrences, about 108 prose
lines, four `Display` strings, and three `unreachable!` messages.

**voxsmith.** About 230 id-suffix lines across 27 files, six of which hold
most of the weight (`internal/vmax/write_vmax.rs`, `reduce_palette.rs`,
`convert/vmax/to_vmax_file.rs`, `convert/voxelize/voxelize_mesh.rs`,
`internal/gltf/bake_atlas.rs`, `convert/vmax/from_vmax_file.rs`). About 25
index lines plus a 25-line `_idx` cluster in `write_vmax.rs`. About 185
value-pool lines across 29 files, anchored by the identifier renames
(`pool_color` and its file, `PoolColumn`, `property_pool`, `pool_scalar`,
`pool_flag`, two `float_pool`s, `srgb_pool`, `srgba_pool`, `pool_len`,
`numbered_pool`, `non_color_pool`, `pool_ref`). One public field is
breaking: `MaterialMeshRequest::layer` holds a `U32Id<BVoxLayer>`. Seven
sites name a `U32Id<BVoxValuePoolValue>` `index`, the most misleading names
in the crate. Two failure messages say bare "pool".

**vxl.** About 300 id-suffix lines concentrated in 8 files while 108 of 118
files are clean; `implementation/hierarchy_show.rs` alone holds about 155,
mostly treegrid node ids. About 45 index lines, including
`Mesh::layer`, whose field name derives the `--layer` flag, so the flag
follows the rename. About 81 value-pool lines with exactly one user-visible message
(`mesh_object.rs`, "bound to a string or json pool"). The report commands
(`palette_show`, `palette_list`, `info`, `hierarchy_show`, `validate`)
assert exact rendered output in their tests, a safety net: none of those
assertions may change outside a dedicated message commit.

**voxj and voxj-codec** (iteration 4): clean on value pool, zero bare
mentions, with the wire keys `valuePool` and `valuePools` as the sanctioned
short forms. The `_index` sweep missed one shape: the outer subject index of
the seven `check_*` functions, 14 sites, bare `index` while every nested
index is suffixed. Q3 folds them into this plan.

## What is settled

- Iteration order is `voxcore`, then `voxsmith`, then `vxl`, following the
  dependency direction: each crate's breaking renames land before its
  dependents' iterations, so the later iterations stay self-contained.
- An iteration touches sibling crates only where the compiler forces it: the
  call sites of a renamed API land with the rename, taking only the new API
  names, and the sibling's own bindings wait for its iteration.
- Renamed `format!` placeholders keep rendered text byte-identical; message
  text changes are observable output and land in their own commit at the
  end of each iteration.
- `vxl`'s `Mesh::layer` rename lets the derived flag follow to
  `--layer-index`; CLI stability is not a constraint here, and the commit
  body notes the moved flag.
- Wire-fixed names stay: the voxj wire keys, and the Voxel-Max wire field
  names behind `write_vmax.rs` (`material_idx`, `color_idx` on the wire
  structs). The voxsmith-side locals and maps around them do not inherit
  the exception.
- The format spec doc does not change.

## Decisions

### Q1. Bare id-pool prose in voxcore

**Decision (2026-07-29): expand it to id-pool forms.** After the value-pool
sweep, the only bare "pool" mentions left in `voxcore` are the
branded-id-pool ones ("Compact the palette pool", "the voxel pool"). They
become "the palette id pool" and kin in a docs-only commit at the end of
iteration 1, so a bare-pool grep over the workspace returns nothing and the
two entities cannot be confused again.

### Q2. Bare `id` bindings

**Decision (2026-07-29): bare `id` survives only as the subject parameter
of a single-entity function** (`fn palette(&self, id: U32Id<BVoxPalette>)`).
Every loop binding, local, destructure, and closure argument names its
entity (`for (palette_id, palette) in ...`, `.map(|(node_id, _)| ...)`).
This matches what the sweep landed and fixes the sites that read wrong at a
distance.

### Q3. The voxj-codec outer indices

**Decision (2026-07-29): pick them up.** A fourth, small iteration renames
the 14 outer subject indices in the `check_*` functions (`for (index,
node)` becomes `node_index`, and kin) in one commit, closing the last known
`_index` gap.

## Execution shape

1. Iteration 1, `voxcore`: the breaking surface first (with the forced
   voxsmith and vxl call-site follows), then parameters and internals, then
   tests, then prose, then the message commit, then the id-pool prose
   commit (Q1).
2. Iteration 2, `voxsmith`: the breaking `MaterialMeshRequest` field first
   (with the forced vxl follows), then the value-pool identifier renames,
   the id and index sweeps, prose, and the message commit.
3. Iteration 3, `vxl`: the id sweep, the index sweep, the value-pool
   renames, prose, and the message commit.
4. Iteration 4, voxj-codec: the outer subject indices, one commit.

Run the iterations in order; the workspace stays green at every commit.
