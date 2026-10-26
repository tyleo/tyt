# Modeling API

`vxl sdf-doc build` records a model file as an `.sdfj` document.
`vxl sdf-doc voxelize` samples the document on a voxel grid and writes voxj.

This section sets how vxl runs a model and how the model lands on the grid. The
[TypeScript API](#typescript-api) lists the calls a model file can make.

## Running

```sh
# table.sdfj, then table.voxj at 2.5 cm per voxel
vxl sdf-doc build table.ts
  --library materials
vxl sdf-doc voxelize table.sdfj
  --voxel-size 0.025
  --report
```

`vxl sdf-doc build` runs the model file and writes its steps and parts as an
`.sdfj` document. An edit to the model file takes effect at the next build.

| Build flag             | Effect                                                                                       |
| ---------------------- | -------------------------------------------------------------------------------------------- |
| `--library <library>`  | Each one adds a library the model reads, such as vxl's [`materials`](#the-materials-library) |
| `--var <name>=<value>` | Each one sets a build variable that `vars` reads as a JSON value                             |

`vxl sdf-doc voxelize` samples the document on a [voxel grid](#the-grid) and
writes a `.voxj` document:

1. `--voxel-size <meters>` sets the edge of a cell and defaults to 1 meter
2. `--resolution <reference> <n>` sets the voxel size to a reference side of the
   model divided by `n`. `longest-world 32` fits 32 cells across the model's
   widest side
3. `--fill-mode` defaults to `solid`, which keeps every filled cell.
   `--fill-mode surface` keeps only the cells that show: a hollow shell one cell
   thick, plus the opaque cells seen through glass or water
4. `--frame` defaults to `world`, which samples every part on one lattice.
   `--frame local` samples each part in its own frame
5. `--flatten` keeps each joint's node and voxelizes the joint unrotated. Every
   other part voxelizes at its pose on the grid of the joint or root part it
   flattens into
6. `--report` prints the [report](#report) and writes it to `table-report.txt`

### Profiles

A `.vxlconfig` can save either command's flags under a profile name for
`--profile`. A `.vxlconfig` can also give a built `.sdfj` document a library
name for `--library`.

[Model evaluation](https://github.com/tyleo/tyt/blob/main/doc/ref/sdf-doc/model-evaluation.md#settings)
sets how profiles and libraries load.

## The grid

`vxl sdf-doc voxelize` samples the model on a lattice of cubes one voxel size
across. The cell corners sit on whole multiples of the voxel size.

A cell lies inside a shape when the shape's signed distance reads zero or less
at the cell's center.

1. A box with corners on multiples of the voxel size fills exactly the cells
   between them. At `--voxel-size 0.025`, `box([0, 0, 0], [0.05, 0.25, 0.05])`
   fills 2 x 10 x 2 cells
2. A shape centered on a cell corner spans an even number of cells. A shape
   centered on a cell center spans an odd number. A model symmetric about
   `x = 0` therefore gives its center-line features even widths
3. The model's `[0, 0, 0]` lands on the document's root node. A model centered
   on x and z with its base at `y = 0` stands on its pivot
4. Real sizes go in as they are: a seat 45 cm up sits at `y = 0.45`
5. Details sized to the grid take multiples of one voxel size held in a constant
   such as `const v = 0.025;`. The model then reads best at
   `--voxel-size 0.025`

## The materials library

vxl defines the `materials` library with these materials:

| Group  | Names                                                                   | Traits                                                                            |
| ------ | ----------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| Wood   | `bark`, `birch`, `ebony`, `mahogany`, `oak`, `pine`, `walnut`           |                                                                                   |
| Stone  | `brick`, `clay`, `granite`, `marble`, `sandstone`, `slate`, `stone`     |                                                                                   |
| Metal  | `brass`, `bronze`, `copper`, `gold`, `iron`, `silver`, `steel`          | Fully metallic                                                                    |
| Gem    | `amethyst`, `diamond`, `emerald`, `ruby`, `sapphire`, `topaz`           | Smooth and saturated but for a clear `diamond`. Transmit light at their own `ior` |
| Nature | `bone`, `dirt`, `grass`, `ice`, `leaf`, `moss`, `sand`, `snow`, `water` | `ice` and `water` transmit light at their own `ior`                               |
| Craft  | `leather`, `rope`, `straw`                                              |                                                                                   |
| Light  | `ember`, `flame`, `glow`, `lava`                                        | `glow` glows cool white and the rest orange                                       |

Velvet, painted wood, and anything else the library lacks take a custom
`material`.

`vxl palette show table.voxj --property '*' walkable value auto` lists the
custom property `walkable` per material.

### Glow

A material's glow in the review renders follows its `emissiveStrength`:

| `emissiveStrength` | Glow                                    |
| ------------------ | --------------------------------------- |
| 1                  | Glows at the material's `emissiveColor` |
| 2 to 4             | Adds a halo and pales the core          |
| Higher             | Whitens the core                        |

The lit `baseColor` adds to the glow. A dark `baseColor` keeps a strength-1 glow
at its full hue.

## Report

`--report` prints a line for the model, then a line per step in list order. The
[example](#example) table reports:

```
table.voxj  2768 voxels of 0.025 m  1 piece  22x22x22  [-0.275, 0, -0.275] .. [0.275, 0.55, 0.275]
  add    legs       480 cells  360 kept  292 exposed  4 pieces  20x20x20  [-0.25, 0, -0.25] .. [0.25, 0.5, 0.25]                     lost to aprons, sides, shelf
  add    aprons     384 cells  312 kept  308 exposed  2 pieces  16x6x18   [-0.2, 0.35, -0.225] .. [0.2, 0.5, 0.225]                  lost to sides, sapphires
  add    sides      384 cells  384 kept  356 exposed  2 pieces  18x6x16   [-0.225, 0.35, -0.2] .. [0.225, 0.5, 0.2]
  add    shelf      648 cells  648 kept  640 exposed            18x2x18   [-0.225, 0.1, -0.225] .. [0.225, 0.15, 0.225]
  add    top        968 cells  800 kept  672 exposed            22x2x22   [-0.275, 0.5, -0.275] .. [0.275, 0.55, 0.275]              lost to gilt
  paint  gilt       168 cells  168 kept  168 exposed            22x2x22   [-0.275, 0.5, -0.275] .. [0.275, 0.55, 0.275]    over top
  add    sapphires   96 cells   96 kept   48 exposed  3 pieces  14x4x4    [-0.175, 0.375, 0.175] .. [0.175, 0.475, 0.275]
check  4 lines
  add  legs       480 cells  360 kept  292 exposed  4 pieces  20x20x20  [-0.25, 0, -0.25] .. [0.25, 0.5, 0.25]           lost to aprons, sides, shelf
  add  aprons     384 cells  312 kept  308 exposed  2 pieces  16x6x18   [-0.2, 0.35, -0.225] .. [0.2, 0.5, 0.225]        lost to sides, sapphires
  add  sides      384 cells  384 kept  356 exposed  2 pieces  18x6x16   [-0.225, 0.35, -0.2] .. [0.225, 0.5, 0.2]
  add  sapphires   96 cells   96 kept   48 exposed  3 pieces  14x4x4    [-0.175, 0.375, 0.175] .. [0.175, 0.475, 0.275]
```

| Field     | Counts                                                                              |
| --------- | ----------------------------------------------------------------------------------- |
| `cells`   | The cells the step wrote as it ran                                                  |
| `kept`    | The cells no later step changed                                                     |
| `exposed` | The kept cells that show                                                            |
| `pieces`  | The face-connected pieces the step's cells form, shown when there are more than one |
| Size      | Cells along x, y, and z                                                             |
| Bounds    | Meters from the min cell's min corner to the max cell's max corner                  |
| `lost to` | The later steps that changed the step's cells                                       |
| `over`    | The steps whose cells a `paint` or a `coat` recolored                               |

A step's cells hold every cell an `add` or a `set` filled, every live cell a
`carve` emptied, and every live cell a `paint` or a `coat` recolored.

A cell shows when it touches empty space across a face, or when it is opaque and
touches a material that lets light through. A `carve` line has no `exposed`
count.

A cell a later step recolored still counts toward `pieces`, and a cell a later
step emptied does not.

A step's size and bounds cover the cells it wrote. The bounds can feed a `box`
directly.

A step with `0 cells` wrote nothing. An `add` or a `set` fell between the cell
centers, or a `carve`, `paint`, or `coat` met no live cell. A step with cells
and `0 kept` lost them all to the steps after `lost to`.

`--fill-mode surface` thins the voxel counts on the model, part, and piece lines
to the shell the document holds. The other fields read the cells before the
fill.

The example table reads this way:

1. The legs read 4 pieces, one per leg
2. The top keeps 800 of its 968 cells because the gilt takes the rest
3. The sapphires keep all 96 cells and show 48 of them because half of each
   sapphire sits inside the apron

### Parts in the report

1. Each place of a part gets a part line with the part's pivot
2. The part's steps and child parts follow, indented one level past the part
   line
3. A part line counts the part's live cells
4. A part reads `detached` when neither it nor a part below it shares a
   face-connected piece with its parent
5. A part in several places lists its steps under its first place. Each later
   place's line reads `repeats` and the first place's path
6. A part reading `0 voxels` writes its node with no object
7. A step name that repeats across lists takes its path of part names, such as
   `tree.1/tree/trunk`

The model's line and the pieces count the cells the parts cover together.

### Piece lines

The report for a model in more than one face-connected piece ends with a line
per piece. The piece lines run from the largest piece down and list the steps
whose live cells reach each piece.

With its hands at `x * 3.25`, the [figure](#parts) leaves a one-voxel gap at
each wrist:

```
figure.voxj  392 voxels of 0.025 m  3 pieces  28x10x4  [-0.35, 0.3, -0.05] .. [0.35, 0.55, 0.05]
  part  body  320 voxels  8x10x4  [-0.1, 0.3, -0.05] .. [0.1, 0.55, 0.05]  pivot [0, 0.3, 0]
    add  torso  320 cells  320 kept  224 exposed  8x10x4  [-0.1, 0.3, -0.05] .. [0.1, 0.55, 0.05]
    part  arm.L  32 voxels  8x2x2  [0.075, 0.5, -0.025] .. [0.275, 0.55, 0.025]  pivot [0.1, 0.525, 0]
      add  L arm  32 cells  32 kept  32 exposed  8x2x2  [0.075, 0.5, -0.025] .. [0.275, 0.55, 0.025]
      part  hand.L  8 voxels  2x2x2  [0.3, 0.5, -0.025] .. [0.35, 0.55, 0.025]  pivot [0.325, 0.525, 0]  detached
        add  L hand  8 cells  8 kept  8 exposed  2x2x2  [0.3, 0.5, -0.025] .. [0.35, 0.55, 0.025]
    part  arm.R  32 voxels  8x2x2  [-0.275, 0.5, -0.025] .. [-0.075, 0.55, 0.025]  pivot [-0.1, 0.525, 0]
      add  R arm  32 cells  32 kept  32 exposed  8x2x2  [-0.275, 0.5, -0.025] .. [-0.075, 0.55, 0.025]
      part  hand.R  8 voxels  2x2x2  [-0.35, 0.5, -0.025] .. [-0.3, 0.55, 0.025]  pivot [-0.325, 0.525, 0]  detached
        add  R hand  8 cells  8 kept  8 exposed  2x2x2  [-0.35, 0.5, -0.025] .. [-0.3, 0.55, 0.025]
  piece 1  376 voxels  22x10x4  [-0.275, 0.3, -0.05] .. [0.275, 0.55, 0.05]  from torso, L arm, R arm
  piece 2    8 voxels  2x2x2    [-0.35, 0.5, -0.025] .. [-0.3, 0.55, 0.025]  from R hand
  piece 3    8 voxels  2x2x2    [0.3, 0.5, -0.025] .. [0.35, 0.55, 0.025]    from L hand
check  2 lines
  piece 2  8 voxels  2x2x2  [-0.35, 0.5, -0.025] .. [-0.3, 0.55, 0.025]  from R hand
  piece 3  8 voxels  2x2x2  [0.3, 0.5, -0.025] .. [0.35, 0.55, 0.025]    from L hand
```

A second piece usually means a shape floats.

### The check block

The report ends with a `check` line. Under it, the report repeats the lines a
review has to clear:

1. Each step line at `0 kept`
2. Each step line at `0 exposed`
3. Each step line with `pieces`
4. Each piece line past the first

A step line there takes the step's path when the step's name appears in several
lists.
A report with nothing to clear ends at `check  0 lines`.

## Errors

Each command stops at its first error:

| Command  | The error reports                                           |
| -------- | ----------------------------------------------------------- |
| Build    | The failing call, the argument, and the value               |
| Voxelize | The failing value and the failing step's path of part names |

A step that writes no cell stops nothing and reads `0 cells` in the report.

The
[checks](https://github.com/tyleo/tyt/blob/main/doc/ref/sdf-doc/model-evaluation.md#checks)
list every error.

## Resolution

Voxel models run from 16 to several hundred voxels across. At any size the grid
shapes the smallest features:

1. A shape thinner than a voxel can miss every cell center and vanish
2. A feature one voxel thick holds only on whole cells along the axes. A
   one-voxel post takes the `box` from `[x, y0, z]` to `[x + v, y1, z + v]`,
   with `x` and `z` on multiples of `v`
3. A tilted or curved member, a stroke, and a torus tube need about 2 voxels
   across or they break into pieces
4. A sphere, cylinder, cone, or rounded corner under a radius of about 3
   voxels, or an octahedron under about 5, reads as a plus sign, a rod, or a
   block. Small round details read better as boxes or `set` cells
5. A large `add` placed after a detail buries it because later steps win. Large
   forms go first and details after
6. Nested shapes of one feature bury each other in any order
7. Copies from `mirror` and `repeat` share one
   [pattern frame](#pattern-frames). A distinct look per copy takes its own step
   with its own seed
8. A model holds its details at the voxel size they were sized for. A coarser
   sampling can drop details thinner than its cells. A finer one draws each
   detail with more cells without adding any
9. A thin form or a whole object turned off the axes samples into ribs and stair
   steps. Furniture, plates, and dishes read cleanest square to the axes

## Recipes

Each recipe shows shapes as constants and steps as entries of the list, sized
for `--voxel-size 0.025`.

```ts
// Four turned legs from one profile.
const leg = lathe([[0.0375, 0], [0.03, 0.15], [0.045, 0.225], [0.03, 0.425]]).translate([0.2, 0, 0.2]);
add("legs", leg.mirror("xz"), grain(mat.oak, 0.05, { axis: "y", seed: 1 })),

// A jewel set into a surface. The later add takes the cells the two share.
add("jewel", octahedron([0, 0.875, -0.175], 0.075), mat.ruby),

// A gilded top edge.
paint("gilt", intersect(rail.shell(0.025), halfSpace("+y", 0.925)), mat.gold),

// An open bowl.
add("bowl", subtract(sphere([0, 0.15, 0], 0.15).shell(0.025), halfSpace("+y", 0.2)), mat.clay),

// Twelve studs around a rim.
add("studs", box([0.225, 0.075, -0.025], [0.275, 0.125, 0.025]).repeatPolar("y", 12), mat.iron),

// Floor planks in alternating shades.
add("floor", box([-0.4, 0, -0.4], [0.4, 0.025, 0.4]), bands(shades(mat.pine), 0.1, { axis: "x" })),

// A cobbled wall with an arched doorway through it.
add("wall", box([0, 0, 0], [0.8, 0.4, 0.075]), cells(shades(mat.stone), 0.15, { seed: 7, border: { material: mat.dirt, width: 0.025 } })),
carve("doorway", extrude(arch([0.325, 0], [0.475, 0.25]), { from: 0, to: 0.075 })),

// A tapered branch.
add("branch", roundCone([0, 0.25, 0], [0.15, 0.45, 0.05], 0.05, 0.025), mat.bark),

// A lantern cage.
add("cage", boxFrame([-0.1, 0, -0.1], [0.1, 0.25, 0.1], 0.025), mat.iron),

// Snow on every upward face.
coat("snow", mat.snow, 0.05, { sides: ["+y"] }),

// A rough boulder.
add("boulder", sphere([0, 0.125, 0], 0.15).displace({ amplitude: 0.0375, scale: 0.1, seed: 3 }), mat.granite),
```

## Example

```ts
// A walnut side table facing +z with three sapphires set in its front apron,
// sized for --voxel-size 0.025. Building each board around the origin and
// moving it into place centers its grain.
const walnut = (axis: Axis, seed: number) =>
  grain(mat.walnut, 0.05, { axis, seed });

const leg = lathe([
  [0.0375, 0],
  [0.03, 0.15],
  [0.045, 0.3],
  [0.03, 0.5],
]).translate([0.2, 0, 0.2]);
const apron = box([-0.2, -0.075, -0.025], [0.2, 0.075, 0.025]).translate([
  0, 0.425, 0.2,
]);
const side = box([-0.025, -0.075, -0.2], [0.025, 0.075, 0.2]).translate([
  0.2, 0.425, 0,
]);
const shelf = box([-0.225, 0.1, -0.225], [0.225, 0.15, 0.225]);
const top = box([-0.275, -0.025, -0.275], [0.275, 0.025, 0.275], {
  round: 0.025,
}).translate([0, 0.525, 0]);
const rim = subtract(
  top.shell(0.025),
  box([-0.25, 0.5, -0.25], [0.25, 0.55, 0.25]),
);
const sapphires = octahedron([-0.125, 0.425, 0.225], 0.075).repeat(
  [0.125, 0, 0],
  [3, 1, 1],
);

export default [
  add("legs", leg.mirror("xz"), walnut("y", 1)),
  add("aprons", apron.mirror("z"), walnut("x", 2)),
  add("sides", side.mirror("x"), walnut("z", 3)),
  add("shelf", shelf, walnut("x", 4)),
  add("top", top, walnut("x", 5)),
  paint("gilt", rim, mat.gold),
  add("sapphires", sapphires, mat.sapphire),
];
```
