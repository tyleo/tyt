# Writer strictness

Status: **open**, drafted 2026-10-20. Nothing is built yet. An audit of every
format writer found content that drops, clamps, or rounds without an error,
which the [Errors](../../../../CLAUDE.md#errors) rule forbids. This plan makes
each one an error, or writes it exactly where the format can hold it.

Paths are relative to `projects/`. Line numbers are as of `c1b20614`. The
audit read the code without running it. Each step starts with a test that
shows its finding. A finding the test disproves drops out.

## Rule

A writer writes what its format holds exactly. On anything else it errors
with what does not fit and where. It never drops, clamps, rounds, or
approximates. A `vxl` command repairs the document, as
[palette material tools](../palette-material-tools/README.md) plans for
Voxel Max.

Each bridge has two writing arrows. `to_x_file` writes a state already shaped
for its format. `to_x_vox_main` reshapes a scene to fit. Sections 1 and 2
cover the first and section 3 the second.

## 1. Wrong or Invalid Output

These write a file that is invalid or that reads back differently. Each is a
bug whatever question 1 decides.

| Writer        | Finding                                                                                                                                                 | Code                                                                                                                    |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| mvox          | A voxel with no color takes slot 0, the reserved empty index. `to_mvox_bytes` never calls `validate_mvox_file`, which would reject it                   | `mvox-voxcore/src/to_mvox_vox_main.rs:118-121`, `to_mvox_file.rs:313-317`, `mvox-codec/src/to_mvox_bytes.rs:7-11`       |
| mvox          | `to_mvox_file` never reads an object's origin. An origin changed after load may then shift voxels                                                       | `mvox-voxcore/src/to_mvox_file.rs:296-331`                                                                              |
| qb, qbt, qbcl | No writer reads an object's origin. The vmax and voxj loaders set origins, so converting from them shifts voxels                                        | `qbcl-voxcore/src/to_qb_file.rs:148`, `internal/fold_under_root.rs`                                                     |
| vmax          | A material binding `emissiveColor` without `emissiveStrength` writes strength 0. voxcore's default is 1                                                 | `vmax-voxcore/src/to_vmax_file.rs:914-916`, `voxcore/src/material/consts.rs:41`                                         |
| vmax          | Two materials sharing a color cell on different slots both set their bits in `lc`. Voxel Max reads a voxel's material from `lc`, so one comes out wrong | `vmax-voxcore/src/to_vmax_file.rs:1486-1506`                                                                            |
| vmax          | A node placing both objects and child nodes, in a state `to_vmax_vox_main` did not shape, gives its children an object for a parent                     | `vmax-voxcore/src/to_vmax_file.rs:129-142, 318-321`                                                                     |
| goxl          | An object no layer stamps writes a block nothing draws                                                                                                  | `goxl-voxcore/src/to_goxl_file.rs:37-41`                                                                                |
| glTF          | A node with several parents writes as one, which glTF forbids                                                                                           | `voxsmith/src/operations/object/object_mesh/write/write_hierarchy.rs:57-62`, `gltf-meshdoc/src/to_gltf_file.rs:604-609` |
| glTF          | An empty object can leave `primitives: []`, which glTF forbids                                                                                          | `gltf-meshdoc/src/to_gltf_file.rs:350-352, 559-573`                                                                     |
| glTF          | Node transforms narrow to f32 with no range check                                                                                                       | `gltf-meshdoc/src/to_gltf_file.rs:1139-1149`                                                                            |
| voxj          | Ids that `VoxMain::gc` has not compacted write as indices, so references can point at the wrong entry                                                   | `voxj-voxcore/src/to_voxj_file.rs:28-31`                                                                                |

The comment at `gltf-meshdoc/src/to_gltf_file.rs:597-598` says a node with
several objects writes one child per object, while lines 626-632 error on it.
The comment is wrong.

## 2. Silent Drops in `to_x_file`

These writers error on the plain limits, such as palette size, grid size, and
layer count in mvox and vmax. They drop the following silently.

| Content                                                              | Writers             | Code                                                                                                                   |
| -------------------------------------------------------------------- | ------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| A color component outside `[0, 1]` clamps                            | every 8-bit writer  | `voxcore/src/color/srgba_u8_from_lin_srgba_f64.rs:7-11`                                                                |
| Layers past the one holding `baseColor`                              | goxl, qb, qbt, qbcl | `voxcore/src/color/resolve_cell_color.rs:8-24`                                                                         |
| Material properties, which the writer rebuilds from the ext alone    | mvox, goxl          | `mvox-voxcore/src/to_mvox_file.rs:229-271`, `goxl-voxcore/src/to_goxl_file.rs:55-66`                                   |
| Material properties and alpha                                        | qb, qbt, qbcl       | `qbcl-voxcore/src/to_qb_file.rs:121-128`, `to_qbt_file.rs:218-220`, `to_qbcl_file.rs:245-247`                          |
| `occlusionStrength` and custom properties, checked and never written | vmax                | `vmax-voxcore/src/to_vmax_file.rs:683-687, 850-881`                                                                    |
| Fully transparent voxels turn opaque, and colorless ones black       | goxl                | `goxl-voxcore/src/to_goxl_file.rs:17-19, 114-138`                                                                      |
| Unplaced objects                                                     | vmax, qbt, qbcl     | `vmax-voxcore/src/to_vmax_file.rs:111-233`, `qbcl-voxcore/src/to_qbt_file.rs:30-61`, `to_qbcl_file.rs:32-78`           |
| Fractional positions round                                           | goxl, qb, qbt, qbcl | `goxl-voxcore/src/ext/goxl_ext.rs:87`, `qbcl-voxcore/src/to_qb_file.rs:148`, `to_qbt_file.rs:82`, `to_qbcl_file.rs:98` |
| Rotation and scale                                                   | qb, qbt             | `qbcl-voxcore/src/to_qb_file.rs:145-150`, `to_qbt_file.rs:91-107`                                                      |
| Parent transforms, rotation, scale, and child links                  | goxl                | `goxl-voxcore/src/to_goxl_file.rs:149-193`, `ext/goxl_ext.rs:204-221`                                                  |
| Unknown layer shape tokens                                           | goxl                | `goxl-voxcore/src/to_goxl_file.rs:229, 237-243`                                                                        |
| Transforms on Group and Shape nodes                                  | mvox                | `mvox-voxcore/src/to_mvox_file.rs:438-482`, `ext/mvox_ext.rs:165-169`                                                  |
| Several parents, several roots, unreached nodes go unchecked         | mvox                | `mvox-voxcore/src/to_mvox_file.rs:346-398`                                                                             |
| A node with several parents writes twice                             | qbt                 | `qbcl-voxcore/src/to_qbt_file.rs:127-133`                                                                              |
| Translation on a node without objects                                | qbt, qbcl           | `qbcl-voxcore/src/to_qbt_file.rs:242-243`, `to_qbcl_file.rs:103-108`                                                   |

`qb` already errors on unplaced objects and child nodes
(`to_qb_file.rs:42-102`). The other writers can follow its checks.

## 3. Reshaping in `to_x_vox_main`

Each synthesizer documents losses its format forces:

1. mvox drops rotation and scale, rounds positions, drops unreached nodes,
   binds only `baseColor`, and flattens layers to it
   (`to_mvox_vox_main.rs:38-40, 89-160, 189, 289`). It also writes a
   zero-size object unchecked (`63-77`)
2. goxl collapses grouping, drops rotation and scale, and rounds positions
   (`to_goxl_vox_main.rs:26-28, 176`)
3. qb collapses grouping, drops rotation, scale, and alpha, and rounds
   positions (`to_qb_vox_main.rs:18-20, 124`)
4. qbt and qbcl drop rotation, scale, and group names, and release unreached
   nodes, which strands their objects at the origin (`to_qbt_vox_main.rs:13-16`,
   `to_qbcl_vox_main.rs:12-15`, `internal/fold_under_root.rs:9-50`)
5. vmax releases unreached nodes, and `to_vmax_file` then drops their objects
   (`to_vmax_vox_main.rs:60, 71-90`). Its doc at line 23 wrongly says it
   loses only the palette name

Tiling a node once per placement keeps every voxel and stays allowed. The
losses above reach the user with no message.

## Questions

1. **Synthesizer losses.** A synthesizer can error on each loss unless the
   caller opts in to that kind of loss, through a flag such as
   `--allow-loss rotation,scale`. It can instead warn. Erroring by default
   keeps the rule. The opt-in keeps conversions to small formats working
2. **Labels.** Object names drop in every writer but voxj and mvox's
   synthesizer. A name is a label, so an error on it would fail most
   conversions. A warning, or carrying the name where the format has a slot,
   may fit better
3. **Colors outside `[0, 1]`.** The writer can error and `vxl palette edit`
   can clamp them on request, or the clamp can stay as the one documented
   quantization
4. **Another format's ext.** When the ext belongs to another format, voxconv
   drops it and synthesizes the file (`voxconv/src/ext/write_with_ext.rs:6-11`).
   That drop may count as a loss under question 1
5. **Repair commands.** Each new error needs a `vxl` command that fixes the
   document, or a note that none exists yet
