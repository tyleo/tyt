# Survey inventory (2026-07-29)

The line-level findings behind the [checklist](../checklist.md). Line
numbers are from 2026-07-29 and drift as edits land; re-grep before editing.
Counts are approximate. Paths are relative to the crate's `src/`.

## voxcore

### Breaking API (iteration 1, first commit)

- `vox_property.rs:13`: `pub pool_id: U32Id<BVoxValuePool>` on
  `VoxProperty`. Read at `vox_main.rs:255, 471, 1036, 1141`,
  `vox_palette.rs:57, 328, 364, 389`, about 25 test sites, and downstream
  at voxsmith `write_vmax.rs:933`, `order_palette_colors.rs:19`,
  `reduce_palette.rs:1195`, `voxelize_mesh.rs:657`,
  `voxj_palette_from_vox_palette.rs:15`.
- `error.rs`: `pool_id` fields on `EmptyValuePool` (15), `UnknownValuePool`
  (40), `PropertyValuePoolRef` (86), `ValuePoolBound` (143),
  `ValuePoolValue` (148), `PropertyValuePool` (156); display arms at 230,
  263, 323, 395, 402, 411.
- `vox_value_pool.rs:284`: `pub fn clone_pool` becomes `clone_value_pool`
  (siblings are `clone_object`, `clone_palette`, `clone_state`); caller at
  `vox_runtime_state.rs:54`.

### Public and pub(crate) parameter names

- `vox_main.rs:334` `pool_id` on `add_property` (doc at 320 to 326),
  `vox_main.rs:404` `pool` on `add_value_pool`, `vox_main.rs:871` `pool_id`
  on `remove_value_pool_value` (doc at 862 to 869), `vox_main.rs:1076`
  `pool_id` on `reorder_value_pool` (doc at 1070 to 1074).
- `vox_palette.rs:44` `pool_id` on `add_property` (doc at 39),
  `vox_palette.rs:313` `pool_id` on `repoint_value_pool_value`.
- `vox_effective_property.rs:27` `pub(crate) pool` field (reads at 49, 76;
  init at `vox_main.rs:508`); the public accessor is already
  `value_pool()`.

### Internal identifiers

- `vox_main.rs`: `pool_ref` (338, 341, 880, 881, 884), `let pool` (377,
  471, 498, 926, 1055, 1136), `pool_id_space` (921, 923), `for pool_id in`
  (924), `pool_remap` (934, 937, 951, 1006), `pool_ids` (1027, 1029,
  1049), `for (pool_id, pool)` (1120).
- `vox_palette.rs`: `pool_property_ids` (323, 331, 336),
  `property_pool_ids` (357, 372), `pool_id` destructure (372).

### Test identifiers (about 180 occurrences)

- Helpers: `vox_main.rs:1373 fn pool_id`, `vox_main.rs:1410 fn int_pool`,
  `vox_main.rs:1424` and `1436` palette helpers taking `pool_id`,
  `vox_palette.rs:415 fn pool_id`, `vox_effective_palette.rs:87 fn
  int_pool`.
- Locals: `pool_a_id`/`pool_b_id` (1983), `first_pool_id`/`second_pool_id`
  (2843), `wild_pool_id` (3357, 3373, 3457, 3490), `let (pool,
  resolved_value_id)` (2028, 2267, 2376, 2383, 2394), `pool_ref` (2457,
  2904, 2911), `let pool` (1650, 2823), `pools` (1484),
  `vox_value_pool.rs:551, 570, 602, 632`, `vox_effective_palette.rs:127,
  151, 165, 185, 202`.
- Test names: `vox_main.rs:2404, 2423, 2841`, `vox_property.rs:22`.
- Debug dump: `vox_main.rs:3180` `format!("|pool {pool_id:?} ...")`; the
  rename changes snapshot text, so it rides the message commit.

### Prose (about 108 lines meaning a value pool)

- `vox_value_pool_value_ref.rs`: all nine variant docs (6 to 30), "A value
  from a `json` pool" and kin.
- `vox_value_pool.rs`: type docs (11, 14), nine constructor docs (30, 41,
  52, 67, 83, 94, 106, 118, 130), accessor docs (142, 154, 235, 240, 245,
  261, 268, 326).
- `error.rs` docs: 21, 51, 66, 90, 142, 159.
- `vox_main.rs` rustdoc and comments: 246, 359, 436, 452, 460, 867, 1022,
  1070, 1074, 1096, 1101, plus internal comments 879, 899, 918 to 943,
  1026, 1047, 1054, 1118, 1131, and about 44 test comment lines.
- `vox_palette.rs`: 101, 133, 275, 349, 355, 381, 383, 515.
- `vox_effective_property.rs`: 27, 47. `vox_value_pool_flaw.rs`: 7, 11.
  `vox_value_pool_kind.rs`: 7. `vox_gc_remap.rs`: 12 (second mention on
  the line). `vox_effective_palette.rs`: 92.

### Messages (dedicated commit)

- `error.rs` `Display`: 286 "not one of the pool's", 306 "each of the
  pool's value ids", 337 "of its pool's", 425 "the pool's values".
- `vox_value_pool.rs` `unreachable!`: 168, 181, 213.

### Id and index residue

- Loop bindings named bare `id` (Q2): `vox_runtime_state.rs:52, 58, 64,
  70`, `vox_object.rs:384`, `vox_main.rs:826`, `vox_value_pool.rs:533`.
- Closure arguments named `id` (about 30 sites) take entity names under
  Q2; the accessor subject parameters (about 35 sites) stay `id`.
- `vox_main.rs:1298` `first_cycle_position`: `start` (1309), `node` (1310,
  1311, 1334) are node indices; callers bind `index` (586) and `position`
  (1268); `for (index, node)` at 570 and the `index: usize` parameter at
  609.

### Id-pool mentions (expand to id-pool forms in the Q1 docs commit)

About 50 lines: `vox_object.rs` (42, 309, 420, 460, 474),
`vox_runtime_state.rs` (15, 19, 25, 31, 37, 48, 91), `vox_gc_remap.rs` (6,
15, 22, 25), `b_vox_voxel.rs:2`, `vox_main.rs` gc section (598, 904, 939,
941, 955 to 979, 1170, 1251, 1278, 3175), `vox_palette.rs` (15, 21, 201,
270, 280, 287, 294, 402), `vox_value_pool.rs` (11, 22, 282, 355, 360, 405,
515).

## voxsmith

### Breaking API (iteration 2, first commit)

- `convert/gltf/material_mesh_request.rs:15`: `pub layer:
  U32Id<BVoxLayer>` becomes `layer_id`; vxl call sites follow.

### Value-pool identifier renames

- `internal/pool_color.rs`: file, function, `internal/mod.rs:16, 59`.
- `internal/vmax/write_vmax.rs`: `property_pool` (928), `pool_scalar`
  (901), `pool_flag` (915).
- `convert/voxelize/voxelize_mesh.rs`: `PoolColumn` (383, `indices` field
  at 388 holds value ids), `srgba_pool` (393), `srgb_pool` (416),
  `float_pool` (439).
- `convert/vmax/from_vmax_file.rs`: `float_pool` (471, returns an id).
- `order_palette_colors.rs`: `pool_ref` (22).
- `internal/resolve_cell_color.rs`: `non_color_pool` (39, 98).
- Test helpers: `reduce_palette.rs:1192 pool_len`,
  `convert/voxj/from_voxj_file.rs:104 numbered_pool`.
- Test names: `from_voxj_file.rs:580`, `bake_atlas.rs:381`,
  `reduce_palette.rs:1322`.
- `pool` and `pool_id` locals, parameters, and closure arguments, about 81
  lines; densest in `internal/pool_color.rs:9`,
  `internal/resolve_cell_color.rs:23, 56, 100`,
  `internal/voxj/vox_value_pool_from_voxj_value_pool.rs:20`,
  `internal/voxj/voxj_value_pool_from_vox_value_pool.rs:22`,
  `internal/voxj/write_voxj.rs:44`, `convert/voxj/from_voxj_file.rs:29`,
  and the `(pool, value_id)` and `(pool, index)` destructures in
  `bake_atlas.rs:143, 162`, `reduce_palette.rs:172, 811, 1005`,
  `voxelize_mesh.rs:635`, `to_mvox_file.rs:128`, `from_gltf_bytes.rs:730,
  741, 1572`, `to_vmax_file.rs:1303, 1313, 1388`.

### Id suffixes, the six hot files

- `internal/vmax/write_vmax.rs` (about 29): `Placement.id` (415), `for
  &root` (455), `id` parameter (468), `for &child` (491, 1277), `for
  &object` (1273), `FoldedRef` fields `palette`, `color`, `materials`
  (628 to 630), `let color` (638, 742, 985, 1016), `let first` (696),
  `for material in iter_materials` (698, 758, 1203), closure `|material|`
  (683, 741), `|(_, property)|` (762), id-typed parameters `palette`,
  `property`, `color`, `material` (813, 903, 917, 930, 984, 1076, 1193),
  `let pool = ...pool_id` (933), `let layer` (947), `|voxel|` (950), `let
  material` (954), `fn suffix(object:)` (1471), bare `|id|` closures (427,
  641, 699, 991, 1204).
- `reduce_palette.rs` (about 31): id-typed parameters `palette`,
  `material`, `property`, `layer`, `object` (32, 56, 168 to 170, 179, 546,
  599, 801, 891, 896, 944), `color_property` (73), `Some(property)` (76,
  1194), `|material|` (78), `Point.material: u32` and `representative()`
  (156), `for (layer, referenced)` (183), `for voxel` (188, 625),
  `Some(material)` (189), `layers`/`layer` (568 to 578, 616), `voxels`
  (612), `let material` (629), `neighbor: u32` closure argument holding a
  voxel id (707); tests: `fn value` (785), `add_value_pool` locals `base`,
  `tag`, `strength` (839, 842, 915, 1237, 1286), `materials` (858, 923,
  1250), `let (mut state, palette, [object])` at 11 sites (957, 976, 1016,
  1042, 1059, 1080, 1120, 1162, 1206, 1325), `let voxel` (932, 950, 1001,
  1258).
- `convert/vmax/to_vmax_file.rs` (about 47, nearly all tests): `let
  palette = add_rgba_palette(...)` at 20 sites (516 to 1723), index
  closures `material`, `object`, `node` (356, 382, 383, 567, 585, 586),
  `node` parameter (432), `for &object` (443), `for &child` (462), `for
  &root` (468), `for voxel` (451), `add_value_pool` locals (306, 626,
  1246 to 1249, 1343), `fn color_object(palette:)` (644), `let id =
  voxel_id(...)` (653, 1201), `fn object_node(object: u32)` (671), `fn
  group_node(children: &[u32])` (681), `let voxel` (329, 364, 376, 574,
  898, 1270, 1364), `let material` (1297, 1384), `let property` (1302,
  1312, 1385), `let (pool, index)` where `index` is a value id (1303,
  1313, 1388).
- `convert/voxelize/voxelize_mesh.rs` (about 16): `default_material` and
  `samples` (70, 376), `Some(material)` (87), `let voxel` (88), eight
  `let *_pool = add_value_pool` (290 to 297), `materials` (352),
  `PoolColumn.indices` (388); tests 628 to 657.
- `internal/gltf/bake_atlas.rs` (about 22): `let material`, `let palette`,
  `let property` (121 to 123); tests: `fn value` (248), eight
  `add_value_pool` locals (268 to 483), nine `add_material` locals (293 to
  495), `let layer` and `let voxel` fixtures (304 to 501), `let (state,
  object_id, layer)` (316 to 457).
- `convert/vmax/from_vmax_file.rs` (about 15): `FoldedPalette.palette`
  (236), `combos` map holding material ids (242), `let material` (217),
  nine `add_value_pool`/`float_pool` locals (272 to 375), `let id =
  add_material` (402), `let palette = add_palette` (408).

### Id suffixes, the converter family and helpers

- `order_palette_colors.rs`: `palette` parameter (12), `color` (16), `let
  Some(pool)` (19), `for material` (31); tests 61 to 108.
- `internal/gltf/used_materials.rs`: fields `palette` (18), `materials`
  (21), accessors `palette()` (30), `material()` (36), parameters `voxel`
  (47), `layer` (58), locals 60 to 70.
- `convert/qbcl/from_qbcl_file.rs`, `from_qbt_file.rs`, `from_qb_file.rs`,
  `from_goxl_file.rs`: the shared shape: `(palette, materials)` returns,
  `palette`/`materials` parameters, `let pool = add_value_pool`, `let
  material = add_material`, `let id = add_hierarchy_node` and `let id =
  voxel_id`, test fixtures.
- `convert/qbcl/to_qbcl_file.rs`: `roots`/`root` (42, 43), `id` parameter
  (82), `|&child|` (146), `|&root|` (249), `for (id, object)` (253),
  `(name, child_objects, child_nodes, world)` (295), `|&id|` (305);
  `to_qbt_file.rs` same shape (43, 44, 68, 140).
- `convert/mvox/to_mvox_file.rs`: `palette` parameter (113, 115),
  `Some(property)` (118), `let material` (125), `(pool, value)` closure
  (128), `let layer` (177), `|layer|`/`|material|` (186, 187), `for voxel`
  (305, 346), `|&node|` (384), `for (id, object)` (386),
  `child_nodes`/`child_objects` (419), `|&child|` (434).
- `convert/mvox/from_mvox_file.rs`: `color_pool`, `type_pool`, `pool`
  (106, 143, 166), `palette` parameter (224); tests 707 to 753.
- `convert/goxl/to_goxl_file.rs`: `for &root` (118), `for (id, object)`
  (121), `(name, child_objects, child_nodes, world)` (153), `for child`
  (173), `for voxel` (198), `|&id|`/`|id|` (269 to 271).
- `internal/grid.rs`: `for (_, palette)` (37), `layers` (46), `for voxel`
  (47), `let id = voxel_id` (52), `|&layer|` (57).
- Small files: `convert/gltf/material_atlas.rs` (31, tests 72 to 83),
  `object_to_material_glb.rs` (53 to 80), `object_to_material_gltf.rs`
  (53 to 64), `object_to_glb_bytes.rs:43`, `object_to_gltf_bytes.rs:39`,
  `internal/gltf/material_document.rs:55`,
  `convert/mesh/object_to_mesh_geometry.rs` (109, 180, 316),
  `internal/voxj/vox_palette_from_voxj_palette.rs:79`,
  `vox_object_from_voxj_decoded_object.rs` (52, 91),
  `voxj_decoded_object_from_vox_object.rs` (32, 34),
  `voxj_palette_from_voxj_palette` and hierarchy-node converters' bare
  `|id|` closures, `convert/gltf/from_gltf_bytes.rs` tests (718 to 722,
  1115, 1237).

### Index suffixes and the misnamed ids

- `convert/vmax/from_vmax_file.rs`: `color`/`material` value indices (396,
  397), `for (node, parent)` (744), `parent_node` (746), `node_of_id` map
  (720), `object_refs` (56).
- `convert/mvox/from_mvox_file.rs`: `let child = resolve(...)` (281, 294).
- `reduce_palette.rs`: octree arena `node` (368), `child` (374), `slot`
  (617), test `position` index (931).
- `convert/vmax/to_vmax_file.rs`: material indices in test loops (362,
  650), raw `u32` ids on `object_node`/`group_node` (671, 681).
- `internal/vmax/write_vmax.rs` `_idx` cluster (about 25 lines):
  `material_idx` field and locals (657, 681, 697, 708, 757, 784, 802, 964,
  973, 1207), `idx` (768 to 781), `color_idx` (955, 974). The Voxel-Max
  wire-struct field names stay; the voxsmith-side map and locals become
  `_index` forms.
- Misnamed ids: `let (pool, index)` where `index` is a
  `U32Id<BVoxValuePoolValue>` in `from_gltf_bytes.rs` (730, 741, 1572) and
  `to_vmax_file.rs` (1303, 1313, 1388), and `PoolColumn.indices` (388).

### Prose (about 90 lines)

Densest: `voxelize_mesh.rs` (270 to 288, 382 to 465, 638, 653),
`write_vmax.rs` (662, 665, 729, 808, 899, 913, 1070), `from_mvox_file.rs`
(30, 75 to 78, 120, 159, 924), `from_vmax_file.rs` (251, 284, 468),
`from_voxj_file.rs` (102, 155 to 180, 256, 578, 582, 620, 656, 672),
`bake_atlas.rs` (112 to 159, 262, 382), `reduce_palette.rs` (164, 837,
1190, 1205, 1222, 1324), `order_palette_colors.rs` (27, 81, 127),
`resolve_cell_color.rs` (9, 38), `pool_color.rs` (5 to 8). One-liners:
`internal/vmax/voxel_max_palette.rs:10`, `voxel_max_material.rs:7`,
`voxel_max_material_dispersion.rs:4`, `material_coefficient_scale.rs:6`,
`internal/mvox/magica_voxel_material.rs:5`,
`internal/mesh/mesh_material.rs:9`,
`internal/voxj/vox_palette_from_voxj_palette.rs` (9, 14),
`internal/voxj/write_voxj.rs` (21, 33), `convert/voxj/color_format.rs:2`,
`voxj_file_builder.rs:60`, `to_goxl_file.rs:25`, `to_qbcl_file.rs:27`, the
qb, qbt, qbcl, and goxl readers, `to_mvox_file.rs` (36, 137),
`to_vmax_file.rs` (259, 305, 622, 1240).

### Messages (dedicated commit)

- `order_palette_colors.rs:47` and `resolve_cell_color.rs:41` failure
  texts.

## vxl

### Id suffixes (about 300 lines, 8 files; 108 of 118 files clean)

- `implementation/hierarchy_show.rs` (about 155): the `Scene`/`Walk` walkers
  (`for &root` 150, 240; `for &child` 155, 301; `for &object` 159, 292;
  `roots()` 172; `unplaced_nodes()` 211; `orphan_objects()` 220;
  `enumerate_from(id:)` 267), treegrid `parent` parameters (517, 525, 570, 651,
  752, 800, 898, 947), grid-id locals `leaf`, `section`, `subtree`, `grid_node`,
  `roots`, `unplaced`, `orphans`, `parent` (526 to 953, 687, 821, 542 to 560),
  `Entity::Node(id)`/`Entity::Object(id)` (572, 583), `nodes`/`objects` id vecs
  (595, 601, 617, 622), `child_nodes`/`child_objects` (706, 718, 738, 742),
  bare-`id` parameters on the placement and name helpers (178 to 203, 500, 509,
  629, 638, 650, 799; subject parameters, stay `id` under Q2); test fixtures:
  `let body|head|hand_mesh|...` object ids (1274 to 2044), node-id locals (1275
  to 1894), `let pool` (1332), `first`/`second` palette ids (1348, 1355),
  `let id = voxel_id` (1221); `body_id` at 1366 is the file's own
  counter-example. Note the `NodeId` and `ObjectId` aliases are duplicated with
  `resolve_objects.rs`; hoisting is optional and not required by this plan.
- `implementation/resolve_objects.rs` (about 28): `objects` id vec (42)
  and parameters (70, 146), `for &root` (100), `for &child` (134), `for
  &child_object` (158), test helper `fn object` returning an id (194),
  test locals `a`, `b`, `c`, `root`, `group`, `keep`, `drop` (250 to 316).
- `implementation/info.rs` (about 28): grid-id locals `document`,
  `palettes`, `objects`, `row`, `branch`, `properties`, `node` (77 to
  268), `for (id, palette|object)` (96, 113, 181, 196); tests: `let pool`
  (331), `material`/`glow_material` (336, 499, 506), palette ids (337,
  500, 507), `let voxel` (341, 457), `strengths`/`colors` value-pool ids
  (493, 502).
- `implementation/palette_list.rs` (about 30): grid-id locals `root`,
  `branch`, `row`, `node`, `subtree` (88 to 156, 182), `let count` at 92
  holding a node id, `palette` parameters (168, 177), `for (id, palette)`
  (89, 123); tests: value-pool ids (220, 228, 414, 443), material ids
  (238, 247), palette ids (240, 248).
- `implementation/palette_show.rs` (about 25): `.map(|material| ...)`
  where the argument is a material id (259), grid-id locals `palette`,
  `property`, `data`, `id` (498 to 525); tests: `fn srgba_pool` (627),
  `fn value` (621), value-pool id locals (648 to 1325).
- `implementation/mesh_object.rs` (about 30): `let palette` and closure
  holding a palette id (73 to 77), `palette` parameters (139, 164, 195);
  tests: value-pool ids (347, 387 to 389, 496), `material` (352),
  `palette` (353, 407, 416, 502), `ids` layer vec (365), `fn value` (339).
- `implementation/validate.rs` (about 12): grid-id locals `name_root`,
  `valid`, `root`, `node`, `failures` (88 to 105).
- `implementation/voxelize.rs`: `let palette` and closure (70), `let
  Some((palette, _))` (138).

### Index suffixes (about 45 lines)

- `commands/mesh/mesh.rs`: field `layer: usize` (69) becomes
  `layer_index`, the derived flag following to `--layer-index`; `objects`
  and `object` locals (179, 189 to 205).
- `dependencies.rs` (174, 175) and
  `implementation/dependencies_impl.rs` (119, 120): `object` and `layer`
  parameters become `object_index` and `layer_index`.
- `implementation/mesh_object.rs`: `object` and `layer` index parameters
  (38, 39, 110, 140, 165, 196).
- `implementation/palette_show.rs`: `Collection { palette: usize }` (91,
  doc says "The resolved palette index"), `index` parameters (169, 209),
  enumerate and `PaletteRef::Index` bindings (127, 139), `let channel`
  (334).
- `implementation/hierarchy_show.rs`: `parent: Option<usize>` (270, 386),
  `this` (278), `parents` (340), `instance` (670, 813).
- `implementation/resolve_objects.rs`: enumerate `index` (50, 60),
  borderline; `object_index` is used correctly nearby.
- Format strings that interpolate renamed bindings (`{object}`, `{layer}`,
  `{index}`) keep rendered text identical: `mesh_object.rs` 49, 119;
  `palette_show.rs` 144, 190.

### Value-pool identifiers and prose (about 81 lines; no id pools in vxl)

- `implementation/palette_show.rs` (53 lines): `pool_id` (218), `pool`
  (222), `pool: &VoxValuePool` parameters (278, 306, 326, 363, 376, 393,
  415, 429) and their call sites, `fn srgba_pool` (627), docs and comments
  (102 to 112, 204, 224, 276, 303, 390, 428, 466, 625, 1192, 1225, 1322),
  test names (1244, 1262, 1321).
- `implementation/mesh_object.rs` (15 lines): `pool_id` (218), `pool`
  (222), `fn pool_kind` (231), docs (190, 224, 229, 426), test name (494),
  and the one user-visible message (243): "bound to a string or json
  pool".
- One-liners: `commands/mesh/channel_source.rs` (29, 236, 256),
  `dependencies.rs` (68, 103), `implementation/write_voxj_document.rs`
  (20), `utilities/voxj_encoding_options.rs` (63),
  `utilities/voxj_color_format.rs` (5), `implementation/voxelize.rs` (77),
  test fixtures in `info.rs` (331) and `hierarchy_show.rs` (1332).
- Already correct, leave alone: `commands/mesh/mesh.rs:122` help text,
  `commands/mesh/property_binding.rs:6`,
  `commands/voxelize/palette_reduction.rs:19`, all of `palette_list.rs`.

## voxj-codec (iteration 4)

Outer subject indices in `internal/voxj_validation/`, bare `index` while
every nested index is suffixed:

- Loop bindings: `check_transforms.rs:11` (`node_index`),
  `check_geometry.rs:12` (`object_index`), `check_indices.rs:11`
  (`object_index`), `check_indices.rs:28` (`node_index`),
  `check_value_pools.rs:17` (`value_pool_index`), `check_palettes.rs:12`
  (`palette_index`), `check_edit_state.rs:22` (`object_index`).
- Helper parameters carrying the same value: `check_geometry.rs:41, 72`,
  `check_value_pools.rs:60, 126, 157`, `check_palettes.rs:52, 74`.

The voxj wire-schema fields holding indices (`value_pool`, `layers`,
`child_nodes`, `child_objects`, `root_nodes`, `materials`) are serde-mapped
wire keys and stay as the spec writes them.
