# SDF Json File Format

An `.sdfj` document records a model's calls as JSON. `vxl sdf-doc build` writes
the document, and `vxl sdf-doc voxelize` reads it. The
[modeling API](../../../utilities/vxl/docs/modeling-api.md) lists the calls.
[Model evaluation](../../../../doc/ref/sdf-doc/model-evaluation.md) sets what
each entry computes.

## Structure

```jsonc
{
  "version": 1,
  "shapes3d": [
    /* ... */
  ],
  "shapes2d": [
    /* ... */
  ],
  "materials": [
    /* ... */
  ],
  "shades": [
    /* ... */
  ],
  "patterns": [
    /* ... */
  ],
  "steps": [
    /* ... */
  ],
  "objects": [
    /* ... */
  ],
  "nodes": [
    /* ... */
  ],
  "rootNodes": [
    /* ... */
  ],
  "names": {
    /* ... */
  },
}
```

1. `version` reads 1
2. Each table holds one kind of value a model can share: `shapes3d` the
   `Shape3d`s, `shapes2d` the `Shape2d`s, `materials` the `Material`s,
   `shades` the `shades` calls, `patterns` the `Pattern`s, and `steps` the
   `Step`s
3. `objects` and `nodes` hold the parts. The two tables follow voxj's objects
   and hierarchy nodes
4. An entry references another entry by its index in that entry's table
5. `rootNodes` lists the nodes at the top of the hierarchy
6. `names` gives entries the [names](#names) a model reads them by when the
   document serves as a library

A document holds `version` and can leave out any other key. A reader takes a
missing table or names map as empty, and a writer leaves each empty one out. An
unknown key fails to read.

## Writing

The builder walks the default export depth first and writes each value into its
table after the values it references. The named exports follow in name order. A
value met again reuses its entry, so a shape feeding several steps appears once.
The walk leaves every reference within a table pointing to an earlier entry.

An option the model leaves out stays out of the entry, and vxl applies the
default. The builder writes four shorthands in full:

1. A `set` of one point writes a list of one point
2. A `speckle` with one accent writes a list of one accent
3. A `scale` by one factor writes the factor once per axis
4. A `grain` with one `base` writes the materials of `shades(base)`

The builder writes each number in its shortest round-trip form, and vxl reads
the number back as the same f64. The builder writes an index as a whole number.

## Entries

In every entry but a `shades` entry, an object, and a node, `kind` holds the
name of the call that made the entry. The other keys take the call's parameter
and option names and hold the arguments as JSON. A `Vec2` or a `Vec3` writes an
array, and a shape, material, pattern, or step writes its index. A `?` marks a
key the entry holds only when the model passes it.

### 3D shapes

| `kind`                           | Keys                                                          |
| -------------------------------- | ------------------------------------------------------------- |
| `box`                            | `min`, `max`, `round?`                                        |
| `boxFrame`                       | `min`, `max`, `thickness`                                     |
| `sphere`                         | `center`, `radius`                                            |
| `ellipsoid`                      | `center`, `radii`                                             |
| `cylinder`                       | `a`, `b`, `radius`, `round?`                                  |
| `cone`, `roundCone`              | `a`, `b`, `radiusA`, `radiusB`                                |
| `capsule`                        | `a`, `b`, `radius`                                            |
| `torus`                          | `center`, `ringRadius`, `tubeRadius`, `axis?`, `from?`, `to?` |
| `octahedron`                     | `center`, `radius`                                            |
| `pyramid`                        | `baseCenter`, `width`, `height`                               |
| `halfSpace`                      | `side`, `at`                                                  |
| `extrude`                        | `profile`, `axis?`, `from`, `to`                              |
| `revolve`                        | `profile`, `axis?`, `center?`                                 |
| `lathe`                          | `points`, `axis?`, `center?`                                  |
| `union`, `intersect`             | `shapes`                                                      |
| `subtract`                       | `base`, `cutters`                                             |
| `smoothUnion`, `smoothIntersect` | `radius`, `shapes`                                            |
| `smoothSubtract`                 | `radius`, `base`, `cutters`                                   |
| `translate`                      | `shape`, `offset`                                             |
| `rotate`                         | `shape`, `axis`, `degrees`, `pivot?`                          |
| `orient`                         | `shape`, `from`, `to`, `pivot?`                               |
| `scale`                          | `shape`, `factor`, `pivot?`                                   |
| `mirror`                         | `shape`, `axes`, `center?`                                    |
| `repeat`                         | `shape`, `step`, `count`                                      |
| `repeatPolar`                    | `shape`, `axis`, `count`, `center?`                           |
| `offset`                         | `shape`, `distance`                                           |
| `shell`                          | `shape`, `thickness`                                          |
| `elongate`                       | `shape`, `lengths`, `center?`                                 |
| `twist`                          | `shape`, `axis`, `degreesPerMeter`, `center?`                 |
| `bend`                           | `shape`, `along`, `toward`, `radius`, `pivot?`                |
| `displace`                       | `shape`, `amplitude`, `scale`, `octaves?`, `seed`             |

A transform or a modifier references its receiver as `shape`. `profile`
references an entry in `shapes2d`.

### 2D shapes

| `kind`                           | Keys                                                             |
| -------------------------------- | ---------------------------------------------------------------- |
| `circle`                         | `center`, `radius`                                               |
| `rect`                           | `min`, `max`, `chamfer?`, `round?`                               |
| `ellipse`                        | `center`, `radii`                                                |
| `ngon`                           | `center`, `sides`, `radius`                                      |
| `star`                           | `center`, `points`, `outerRadius`, `innerRadius`                 |
| `polygon`                        | `points`                                                         |
| `polyline`                       | `points`, `width`                                                |
| `spline`                         | `points`, `width?`                                               |
| `arc`                            | `center`, `radius`, `fromDegrees`, `toDegrees`, `width`, `caps?` |
| `sector`                         | `center`, `radius`, `fromDegrees`, `toDegrees`                   |
| `vesica`                         | `a`, `b`, `width`                                                |
| `arch`                           | `min`, `max`                                                     |
| `union`, `intersect`             | `shapes`                                                         |
| `subtract`                       | `base`, `cutters`                                                |
| `smoothUnion`, `smoothIntersect` | `radius`, `shapes`                                               |
| `smoothSubtract`                 | `radius`, `base`, `cutters`                                      |
| `translate`                      | `shape`, `offset`                                                |
| `rotate`                         | `shape`, `degrees`, `pivot?`                                     |
| `scale`                          | `shape`, `factor`, `pivot?`                                      |
| `mirror`                         | `shape`, `axes`, `center?`                                       |
| `repeat`                         | `shape`, `step`, `count`                                         |
| `repeatPolar`                    | `shape`, `count`, `center?`                                      |
| `offset`                         | `shape`, `distance`                                              |
| `shell`                          | `shape`, `thickness`                                             |

### Materials

| `kind`     | Keys              |
| ---------- | ----------------- |
| `material` | `properties`      |
| `shade`    | `shades`, `index` |

1. `properties` holds the material's properties by name. A boolean, a number, a
   string, or an array of numbers writes as itself. `int` writes
   `{ "kind": "int", "value": ... }`, and `json` writes
   `{ "kind": "json", "value": ... }`
2. A `shade` takes the shade at `index` from the `shades` entry it references.
   The darkest shade takes index 0

An entry in `shades` holds `base`, `count`, and `spread?`. The builder writes
`count` even when the model leaves it out because the call returns that many
materials.

```jsonc
// shades(material({ baseColor: "#8A5A2B", lootTier: int(3) }))
"materials": [
  { "kind": "material", "properties": { "baseColor": "#8A5A2B", "lootTier": { "kind": "int", "value": 3 } } },
  { "kind": "shade", "shades": 0, "index": 0 },
  { "kind": "shade", "shades": 0, "index": 1 },
  { "kind": "shade", "shades": 0, "index": 2 },
],
"shades": [{ "base": 0, "count": 3 }],
```

### Patterns

| `kind`     | Keys                                                |
| ---------- | --------------------------------------------------- |
| `bands`    | `materials`, `axis`, `period?`, `warp?`, `seed?`    |
| `grain`    | `materials`, `axis`, `period?`, `warp?`, `seed`     |
| `gradient` | `materials`, `axis`, `from`, `to`, `warp?`, `seed?` |
| `noise`    | `materials`, `scale`, `octaves?`, `seed`            |
| `cells`    | `materials`, `size`, `seed`, `border?`              |
| `speckle`  | `base`, `accents`, `density`, `seed`                |
| `checker`  | `materials`, `size?`                                |

### Steps

| `kind`  | Keys                                                                   |
| ------- | ---------------------------------------------------------------------- |
| `add`   | `name`, `shape`, `material` or `pattern`                               |
| `carve` | `name`, `shape`                                                        |
| `paint` | `name`, `shape`, `material` or `pattern`, `onto?`                      |
| `coat`  | `name`, `material` or `pattern`, `sides?`, `depth`, `within?`, `onto?` |
| `set`   | `name`, `points`, `material` or `pattern`                              |

A step that takes a material or a pattern holds exactly one of the two keys.

### Parts

A part writes one node named after it and, when its list holds steps, one object
of the same name. An object holds `name` and `steps`, the indices of its steps
in list order. Each object voxelizes on its own grid. A node holds `name`,
`pivot?`, `offset?`, `rotation?`, `childObjects`, and `childNodes`. The part's
object goes in `childObjects` and its child parts' nodes in `childNodes`.
`rotation` lists the part's axis rotations in order. Each holds an `axis` and
`degrees`.

```jsonc
// part("tree.1", { offset: [-0.75, 0, 0], rotation: [["y", 30]] }, [tree])
{
  "name": "tree.1",
  "offset": [-0.75, 0, 0],
  "rotation": [{ "axis": "y", "degrees": 30 }],
  "childObjects": [],
  "childNodes": [0],
}
```

A part in several lists writes one node that each parent lists. The builder
writes the default export as one root node named after the model file's stem.

## Names

`names` holds a map per kind of value a model can name. Each map takes a name to
an index. `shapes3d`, `shapes2d`, `materials`, `patterns`, and `steps` index
their tables, and `parts` indexes `nodes`. One entry can take several names.

The builder writes a name for each of the model's named exports, with each map
in name order. An entry the model copies from a
[library](../../../utilities/vxl/docs/modeling-api.md#libraries) keeps every
name it holds there.

```jsonc
// export const walnut = material({ baseColor: "#5C4033" });
// export const wood = walnut;
// export default [];
"materials": [{ "kind": "material", "properties": { "baseColor": "#5C4033" } }],
"names": {
  "materials": { "walnut": 0, "wood": 0 },
},
```

## Rules

vxl reads a document only when the document follows the [structure](#structure),
the [entries](#entries), and these rules:

1. Every index points into its table
2. An entry in `shapes3d`, `shapes2d`, or `nodes` references only earlier
   entries of its own table
3. A shade's `index` stays below its `shades` entry's `count`, and the `shades`
   entry's `base` comes before the shade in `materials`
4. `rootNodes` lists each node at most once, and a root node sits in no
   `childNodes`
5. Every name in `names` points into its table

The [checks](../../../../doc/ref/sdf-doc/model-evaluation.md#checks) cover the
arguments' values.

## Examples

### Chair

The legs and seat of the
[chair](../../../utilities/vxl/docs/modeling-api.md#example):

```ts
const oak = (axis: Axis, seed: number) => grain(mat.oak, { axis, seed });
const leg = lathe([
  [0.0375, 0],
  [0.03, 0.15],
  [0.045, 0.225],
  [0.03, 0.425],
]).translate([0.2, 0, 0.2]);

export default [
  add("legs", leg.mirror("xz"), oak("y", 1)),
  add("seat", box([-0.25, 0.425, -0.25], [0.25, 0.475, 0.25]), oak("x", 2)),
];
```

Built with the `materials` library, `chair.sdfj` holds:

```jsonc
{
  "version": 1,
  "shapes3d": [
    {
      "kind": "lathe",
      "points": [
        [0.0375, 0],
        [0.03, 0.15],
        [0.045, 0.225],
        [0.03, 0.425],
      ],
    },
    { "kind": "translate", "shape": 0, "offset": [0.2, 0, 0.2] },
    { "kind": "mirror", "shape": 1, "axes": "xz" },
    { "kind": "box", "min": [-0.25, 0.425, -0.25], "max": [0.25, 0.475, 0.25] },
  ],
  "materials": [
    {
      "kind": "material",
      "properties": {
        /* mat.oak */
      },
    },
    { "kind": "shade", "shades": 0, "index": 0 },
    { "kind": "shade", "shades": 0, "index": 1 },
    { "kind": "shade", "shades": 0, "index": 2 },
    { "kind": "shade", "shades": 1, "index": 0 },
    { "kind": "shade", "shades": 1, "index": 1 },
    { "kind": "shade", "shades": 1, "index": 2 },
  ],
  "shades": [
    { "base": 0, "count": 3 },
    { "base": 0, "count": 3 },
  ],
  "patterns": [
    { "kind": "grain", "materials": [1, 2, 3], "axis": "y", "seed": 1 },
    { "kind": "grain", "materials": [4, 5, 6], "axis": "x", "seed": 2 },
  ],
  "steps": [
    { "kind": "add", "name": "legs", "shape": 2, "pattern": 0 },
    { "kind": "add", "name": "seat", "shape": 3, "pattern": 1 },
  ],
  "objects": [{ "name": "chair", "steps": [0, 1] }],
  "nodes": [{ "name": "chair", "childObjects": [0], "childNodes": [] }],
  "rootNodes": [0],
  "names": {
    "materials": { "oak": 0 },
  },
}
```

Each `oak` call writes its own `shades` entry. The palette merges the identical
shades when vxl voxelizes. `mat.oak` copies the `oak` entry from the
[`materials`](../../../utilities/vxl/docs/modeling-api.md#the-materials-library)
library with its name.

### Forest

One tree stands in three places, and one rabbit sits under two of the trees:

```ts
const tree = part("tree", {}, [
  add("trunk", cylinder([0, 0, 0], [0, 0.5, 0], 0.05), mat.bark),
  add("crown", sphere([0, 0.65, 0], 0.25), mat.leaf),
]);

const fur = material({ baseColor: "#C8B8A0" });
const rabbit = part("rabbit", {}, [
  add("body", ellipsoid([0, 0.05, 0], [0.05, 0.05, 0.075]), fur),
]);

export default [
  part("tree.1", { offset: [-0.75, 0, 0] }, [
    tree,
    part("rabbit.1", { offset: [0.15, 0, 0.15] }, [rabbit]),
  ]),
  part("tree.2", { offset: [0, 0, -0.5] }, [tree]),
  part("tree.3", { offset: [0.75, 0, 0] }, [
    tree,
    part("rabbit.2", { offset: [-0.15, 0, 0.15] }, [rabbit]),
  ]),
];
```

Built with the `materials` library, `forest.sdfj` holds:

```jsonc
{
  "version": 1,
  "shapes3d": [
    { "kind": "cylinder", "a": [0, 0, 0], "b": [0, 0.5, 0], "radius": 0.05 },
    { "kind": "sphere", "center": [0, 0.65, 0], "radius": 0.25 },
    {
      "kind": "ellipsoid",
      "center": [0, 0.05, 0],
      "radii": [0.05, 0.05, 0.075],
    },
  ],
  "materials": [
    {
      "kind": "material",
      "properties": {
        /* mat.bark */
      },
    },
    {
      "kind": "material",
      "properties": {
        /* mat.leaf */
      },
    },
    { "kind": "material", "properties": { "baseColor": "#C8B8A0" } },
  ],
  "steps": [
    { "kind": "add", "name": "trunk", "shape": 0, "material": 0 },
    { "kind": "add", "name": "crown", "shape": 1, "material": 1 },
    { "kind": "add", "name": "body", "shape": 2, "material": 2 },
  ],
  "objects": [
    { "name": "tree", "steps": [0, 1] },
    { "name": "rabbit", "steps": [2] },
  ],
  "nodes": [
    { "name": "tree", "childObjects": [0], "childNodes": [] },
    { "name": "rabbit", "childObjects": [1], "childNodes": [] },
    {
      "name": "rabbit.1",
      "offset": [0.15, 0, 0.15],
      "childObjects": [],
      "childNodes": [1],
    },
    {
      "name": "tree.1",
      "offset": [-0.75, 0, 0],
      "childObjects": [],
      "childNodes": [0, 2],
    },
    {
      "name": "tree.2",
      "offset": [0, 0, -0.5],
      "childObjects": [],
      "childNodes": [0],
    },
    {
      "name": "rabbit.2",
      "offset": [-0.15, 0, 0.15],
      "childObjects": [],
      "childNodes": [1],
    },
    {
      "name": "tree.3",
      "offset": [0.75, 0, 0],
      "childObjects": [],
      "childNodes": [0, 5],
    },
    { "name": "forest", "childObjects": [], "childNodes": [3, 4, 6] },
  ],
  "rootNodes": [7],
  "names": {
    "materials": { "bark": 0, "leaf": 1 },
  },
}
```

Voxelizing writes a voxj node for each place.
[Model evaluation](../../../../doc/ref/sdf-doc/model-evaluation.md#the-voxj-document)
sets which places share an object. Offsets add up: the rabbit under `tree.1`
lands at `[-0.6, 0, 0.15]` with the path `forest/tree.1/rabbit.1/rabbit`.
