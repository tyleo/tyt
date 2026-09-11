# voxcore

The in-memory voxel model at the bottom of the voxel stack. Each format has a
`-voxcore` bridge crate that reads its files into a `VoxMain` and writes one
back out.

## Architecture

`VoxMain<T>` holds a scene the way every format sees it. Value pools hold the
shared values. Palettes bind named properties to those pools and hold the
materials that draw from them. Objects are dense voxel grids whose layers
reference palettes. Hierarchy nodes place objects and each other under
transforms. `T` is the ext, the format-specific state a bridge carries
alongside the scene.

Each kind of entity lives in an id pool. Ids are branded, so a material id
cannot stand in for a voxel id. Ids are meaningful only within the state that
issued them. Because every mutation checks the cross-references it could
break, a state reached through the public API never violates a referential
rule. `validate` checks a whole state at once.

The sections below build one scene up from a value pool and read it back.

## Value Pools

A `VoxValuePool` is one typed column: `bool`, `float`, `int`, `string`,
`json`, or a vector of two, three, or four floats or ints. The constructor
for a kind takes its values and hands out ids in order.

```rust
let mut state: VoxMain = VoxMain::default();

let colors = VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0]])?;
let colors_id = state.retain_value_pool(colors);
let red_value_id = U32Id::from_u32(0);
```

## Palettes

A `VoxPalette` binds each named property to a value pool. Each material
picks one value id per property. The `material` module fixes the shared
metallic-roughness names, such as `BASE_COLOR` for `baseColor`.

```rust
let palette_id = state.retain_palette(VoxPalette::default())?;
state.retain_property(palette_id, BASE_COLOR.to_owned(), colors_id, red_value_id)?;
let red_id = state.retain_material(palette_id, vec![red_value_id])?;
```

## Objects

A `VoxObject` is a dense grid with a build volume and an origin. Each layer
references a palette. Each live voxel samples one material per layer. Layers
override back to front: a property reads through the last layer whose palette
supplies it. Every cell has a voxel id, which `voxel_id` looks up from a
position.

```rust
let cube = VoxObject::new("cube".to_owned(), TyVector3U32::new(2, 2, 2))?;
let corner = TyVector3U32::new(0, 0, 0);
let corner_id = cube.voxel_id(corner).expect("inside the grid");

let cube_id = state.retain_object(cube)?;
state.retain_layer(cube_id, palette_id, red_id)?;
state.retain_voxel(cube_id, corner_id, &[red_id])?;
```

## Hierarchy

A `VoxHierarchyNode` carries a transform and places child nodes and objects.
Nodes form a DAG: one node can sit under several parents. The scene's roots
are the nodes nothing places.

```rust
let root_id = state.retain_hierarchy_node(VoxHierarchyNode {
    name: "root".to_owned(),
    child_object_ids: vec![cube_id],
    ..Default::default()
})?;
state.push_root_hierarchy_node_id(root_id)?;

state.validate()?;
```

## Reading Back

`effective_palette` resolves an object's layer override rule once. Look a
property up by name, then read voxel values by id.

```rust
let cube = state.object(cube_id).expect("retained above");
let palette = state.effective_palette(cube)?;
let base_color_id = palette
    .property_id_by_name(BASE_COLOR)
    .expect("bound above");

let color = palette.voxel_value(corner_id, base_color_id);
let red = VoxValuePoolValueRef::Vec4Float(&[1.0, 0.0, 0.0, 1.0]);
assert_eq!(color, Some(red));
```

## Releasing and Compaction

A release refuses while anything still references the entity. A release that
goes through leaves a hole in its id pool. `gc` compacts every pool back to
contiguous ids. Call it once before a save to keep saves deterministic. It
returns a `VoxGcRemap` that translates any ids held outside the state.

```rust
let spare_id = state.retain_material(palette_id, vec![red_value_id])?;
state.release_material(palette_id, spare_id)?;

state.gc();
```

## The Ext

Every format keeps state that voxcore has no home for: Voxel Max's material
coefficients and object state, MagicaVoxel's cameras and unknown chunks,
Goxel's lights and previews, Qubicle's thumbnails and metadata. Dropping that
state would make a round trip through voxcore lossy. A bridge's loader stores
its format's ext as the state's `T`, and its writer reads the ext back. A
document loaded from a format writes back to that format exactly. A bare
state writes from the scene. The core never reads the ext.

A format ext aligns its entries with the scene by listing index. A mutation that
moves a listing would leave it stale. The state tells the ext through `VoxExt`,
and every mutation carries that bound. A retain or move fires its hook after the
mutation, and a release fires its hook before it, once every check has passed.
Each carries the index the entity had. The ext drops or inserts its entry in
step. Every hook defaults to a no-op. `()` ignores them all. The hooks are the
whole trait. How an ext persists, and how a boxed one downcasts or clones,
belongs to the crate that converts between formats.

```rust
#[derive(Clone, Debug, Default)]
struct Released(Vec<usize>);

impl VoxExt for Released {
    fn object_will_release(&mut self, index: usize) {
        self.0.push(index);
    }
}

let mut state: VoxMain<Released> = VoxMain::default();
let first_id = state.retain_object(first)?;
state.retain_object(second)?;
state.release_object(first_id)?;
assert_eq!(state.ext().0, [0]);
```

`take_ext` takes the ext off a state, leaving a bare `VoxMain<()>`, and
`put_ext` puts one on a bare state. Together they change a state's ext type
and move the scene over unchanged.

```rust
let (bare, released) = state.take_ext();
let state: VoxMain<Released> = bare.put_ext(released);
```

## Features

The `color` feature adds the `color` module: the sRGB transfer at the 8-bit
boundary and color reads over palettes. It is on by default.
