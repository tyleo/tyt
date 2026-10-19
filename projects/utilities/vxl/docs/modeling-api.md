# Modeling API

A model file describes a voxel model in TypeScript as signed distance functions
measured in meters. Its code runs once and returns a list of steps.
`vxl sdf-doc build` records the steps as an `.sdfj` document.
`vxl sdf-doc voxelize` samples the `.sdfj` document on a voxel grid and writes
voxj. This page lists everything a model file can use and stands alone as the
context for writing one.
[Model evaluation](https://github.com/tyleo/tyt/blob/main/doc/ref/sdf-doc/model-evaluation.md)
sets exactly what each call computes.

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
Each `--library <library>` adds a [library](#libraries) the model reads, such
as vxl's [`materials`](#the-materials-library).

`vxl sdf-doc voxelize` samples the document on a [voxel grid](#coordinates) and
writes a `.voxj` document:

1. `--voxel-size <meters>` sets the edge of a cell and defaults to 1 meter
2. `--resolution <reference> <n>` sets the voxel size to a reference side of the
   model divided by `n`. `longest-world 32` fits 32 cells across the model's
   widest side
3. `--fill-mode surface` keeps only the cells that show: a hollow shell one
   cell thick, plus the opaque cells seen through glass or water. The default
   `--fill-mode solid` keeps every filled cell
4. `--frame local` voxelizes each [part](#parts) once, and every place shares
   the part's object. The default `--frame world` samples every part on one
   lattice and writes an object for each place
5. `--report` prints the [report](#report) and writes it to `table-report.txt`

### Profiles

A `.vxlconfig` can save either command's flags under a profile name for
`--profile`.
[Model evaluation](https://github.com/tyleo/tyt/blob/main/doc/ref/sdf-doc/model-evaluation.md#settings)
sets how profiles and libraries load.

## Model files

A model file is a TypeScript module. Its default export lists the model's steps
and parts. `vxl sdf-doc build` puts every name on this page in scope, and a
model imports none of them:

```ts
export default [add("block", box([0, 0, 0], [0.1, 0.1, 0.1]), mat.stone)];
```

Variables, functions, and loops shape the list, but nothing in a model runs per
voxel. Shapes, steps, and patterns only describe what vxl samples later. A model
holds no voxel size. A variable named like a call on this page hides the call. A
material held in `const paint` breaks every `paint` step.

A larger model can split across files. A file imports another by its relative
path with the `.ts` extension. Every file sees the names on this page.

A named export gives its value a name in the `.sdfj` document. The document can
then serve as a [library](#libraries).

## Coordinates

The axes are glTF's: +Y up, +X right, and +Z toward the viewer. A model faces
+Z. Positions, distances, and radii count meters, and angles count degrees. A
positive rotation follows the right-hand rule: counterclockwise when seen from
the positive end of its axis.

`vxl sdf-doc voxelize` samples the model on a lattice of cubes one voxel size
across. The cell corners sit on whole multiples of the voxel size. A cell lies
inside a shape when the shape's signed distance reads zero or less at the cell's
center.

1. A box with corners on multiples of the voxel size fills exactly the cells
   between them. At `--voxel-size 0.025`, `box([0, 0, 0], [0.05, 0.25, 0.05])`
   fills 2 x 10 x 2 cells.
2. A shape centered on a cell corner spans an even number of cells, and one
   centered on a cell center spans an odd number. A model symmetric about
   `x = 0` therefore gives its center-line features even widths.
3. The model's `[0, 0, 0]` lands on the document's root node. A model centered
   on x and z with its base at `y = 0` stands on its pivot.
4. Real sizes go in as they are: a seat 45 cm up sits at `y = 0.45`. Details
   sized to the grid take multiples of one voxel size held in a constant such as
   `const v = 0.025;`. The model then reads best at `--voxel-size 0.025`.

## Types

```ts
type Vec2 = [number, number];
type Vec3 = [number, number, number];
type Axis = "x" | "y" | "z";
type Axes = "x" | "y" | "z" | "xy" | "xz" | "yz" | "xyz";
type Side = "+x" | "-x" | "+y" | "-y" | "+z" | "-z";
type AxisRotation = [Axis, number];
```

A `Shape3d` holds a 3D region, and a `Shape2d` holds a 2D region in a plane with
axes u and v. A `Material` holds surface properties, a `Pattern` picks a
material per cell, and a `Step` makes one entry of a part's list. A `Part`
voxelizes its steps into an object that turns about a joint. These values never
change: a method returns a new value and leaves its receiver alone. One shape
can therefore feed several steps.

## Steps

```ts
function add(name: string, shape: Shape3d, material: Material | Pattern): Step;
function carve(name: string, shape: Shape3d): Step;
function paint(
  name: string,
  shape: Shape3d,
  material: Material | Pattern,
  options?: PaintOptions,
): Step;
function coat(
  name: string,
  material: Material | Pattern,
  depth: number,
  options?: CoatOptions,
): Step;
function set(
  name: string,
  points: Vec3 | Vec3[],
  material: Material | Pattern,
): Step;

interface PaintOptions {
  onto?: Step[];
}

interface CoatOptions {
  sides?: Side[];
  within?: Shape3d;
  onto?: Step[];
}
```

A part's steps apply in list order to the part's grid, and a later step wins
wherever two reach the same cell.

1. `add` fills the shape's cells with the material and replaces what they held.
2. `carve` empties the shape's cells.
3. `paint` recolors the live cells inside the shape and fills none.
4. `coat` recolors a live cell when the point `depth` meters from its center
   toward a listed side lies outside the part's earlier shapes. `depth` takes
   at least one voxel, and `depth: v` coats the outer layer of cells. `sides`
   defaults to all six. `within` limits the coat to a shape's cells.
5. `set` fills the cell holding each point and places detail too small for a
   shape. A point at a cell center, `(i + 0.5) * v`, lands in that cell. A point
   on a cell boundary can land on either side of it.

`onto` keeps a `paint` or a `coat` on the cells the listed `add` steps own. A
cell belongs to the last `add` that filled it until a `carve` empties it. Each
step in `onto` has to be an `add` earlier in the same list.

Every step takes a name no other step in its list shares. The report and errors
use the name. A part's grid grows to hold every `add` and `set`, and `carve`,
`paint`, and `coat` never grow it.

## Parts

```ts
function part(
  name: string,
  options: { pivot?: Vec3; offset?: Vec3; rotation?: AxisRotation[] },
  list: (Step | Part)[],
): Part;
```

A part groups steps into a voxel object that turns about `pivot`. A part in
another part's list becomes that part's child. Arms, wings, hands, wheels, and
lids take parts when the prompt asks for pieces that move or separate objects.

1. A part's steps run in list order over the part's own grid. A child part
   voxelizes on its own grid, and parts can overlap in space.
2. `carve`, `paint`, and `coat` reach only the cells of their own part. A turned
   joint therefore shows each part whole.
3. The steps and parts outside every part belong to the root part. The root part
   takes the model file's stem as its name and turns about `[0, 0, 0]`.
4. Each part writes its own object under a node at its pivot, and the nodes nest
   as the parts do. Turning a part's node turns the part and its children about
   the joint.
5. `pivot` defaults to `[0, 0, 0]`. `offset` moves the part's steps, child
   parts, and pivot. The offsets of nested parts add up.
6. `rotation` poses the part. Each `[axis, degrees]` in list order rotates the
   part about its pivot on an axis of the parent's frame. The part voxelizes
   unrotated. Its node takes the rotation
7. A step or a part can sit in several lists. A part writes a node at each
   place, and under `--frame local` the nodes share one object.

Every part takes a name no other part in its list shares. A function builds
symmetric parts:

```ts
// A figure whose arms turn at the shoulders and hands at the wrists.
const arm = (side: string, x: number) =>
  part(`arm.${side}`, { pivot: [x, 0.525, 0] }, [
    add(
      `${side} arm`,
      capsule([x, 0.525, 0], [x * 2.5, 0.525, 0], 0.0375),
      mat.leather,
    ),
    part(`hand.${side}`, { pivot: [x * 2.75, 0.525, 0] }, [
      add(`${side} hand`, sphere([x * 3, 0.525, 0], 0.0375), mat.bone),
    ]),
  ]);

export default [
  part("body", { pivot: [0, 0.3, 0] }, [
    add("torso", box([-0.1, 0.3, -0.05], [0.1, 0.55, 0.05]), mat.leather),
    arm("L", 0.1),
    arm("R", -0.1),
  ]),
];
```

One part placed by offsets stands a tree in three spots:

```ts
const tree = part("tree", {}, [
  add("trunk", cylinder([0, 0, 0], [0, 0.5, 0], 0.05), mat.bark),
  add("crown", sphere([0, 0.65, 0], 0.25), mat.leaf),
]);

export default [
  part("tree.1", { offset: [-0.75, 0, 0] }, [tree]),
  part("tree.2", { offset: [0, 0, -0.5] }, [tree]),
  part("tree.3", { offset: [0.75, 0, 0] }, [tree]),
];
```

Another model file pastes the same way. After
`import treeSteps from "./tree.ts";`, `part("tree", {}, treeSteps)` makes that
file's default export one part to place.

## Shapes

### Primitives

```ts
function box(min: Vec3, max: Vec3, options?: { round?: number }): Shape3d;
function boxFrame(min: Vec3, max: Vec3, thickness: number): Shape3d;
function sphere(center: Vec3, radius: number): Shape3d;
function ellipsoid(center: Vec3, radii: Vec3): Shape3d;
function cylinder(
  a: Vec3,
  b: Vec3,
  radius: number,
  options?: { round?: number },
): Shape3d;
function cone(a: Vec3, b: Vec3, radiusA: number, radiusB: number): Shape3d;
function capsule(a: Vec3, b: Vec3, radius: number): Shape3d;
function roundCone(a: Vec3, b: Vec3, radiusA: number, radiusB: number): Shape3d;
function torus(
  center: Vec3,
  ringRadius: number,
  tubeRadius: number,
  options?: { axis?: Axis; from?: number; to?: number },
): Shape3d;
function octahedron(center: Vec3, radius: number): Shape3d;
function pyramid(baseCenter: Vec3, width: number, height: number): Shape3d;
function halfSpace(side: Side, at: number): Shape3d;
```

1. `box` spans its two corners in either order on each axis.
   `round` rounds its edges and corners by that radius inside the same corners.
2. `boxFrame` keeps the twelve edges of the box between its corners as square
   bars `thickness` across that stay inside the corners. Crates, cages, bed
   frames, and lantern frames start from a `boxFrame`.
3. `cylinder`, `cone`, `capsule`, and `roundCone` run from `a` to `b` at any
   angle. A `cone` takes `radiusA` at `a` and `radiusB` at `b`, and a radius of
   0 makes a point. `round` rounds a cylinder's rims. A `roundCone` caps each
   end with a sphere of that end's radius and tapers between the spheres. A
   round cone suits limbs, branches, horns, and bottle necks.
4. A `torus` rings the y axis unless `axis` picks another. `ringRadius` reaches
   from the center to the middle of the tube, and `tubeRadius` sets the tube's
   radius. `from` and `to` cut the ring to an arc with round ends. The angles
   run as an `arc`'s do in the plane `extrude` maps across `axis`. A cut torus
   makes handles, hooks, and horseshoes.
5. `octahedron` reaches `radius` along each axis from its center. From a radius
   of about 3 voxels it makes a cut-gem silhouette, and a smaller one reads as a
   plus sign.
6. `pyramid` stands on a square base `width` across centered on `baseCenter`,
   with its apex `height` above it along +Y.
7. `halfSpace` covers everything past `at` toward `side`:
   `halfSpace("+y", 0.25)` holds every point above `y = 0.25`. A half space has
   no bounds. An `add` takes a half space only inside an `intersect` or as a
   cutter. `carve`, `paint`, and `coat` take one bare.

### 2D shapes

```ts
function circle(center: Vec2, radius: number): Shape2d;
function rect(
  min: Vec2,
  max: Vec2,
  options?: { chamfer?: number; round?: number },
): Shape2d;
function ellipse(center: Vec2, radii: Vec2): Shape2d;
function ngon(center: Vec2, sides: number, radius: number): Shape2d;
function star(
  center: Vec2,
  points: number,
  outerRadius: number,
  innerRadius: number,
): Shape2d;
function polygon(points: Vec2[]): Shape2d;
function polyline(points: Vec2[], width: number): Shape2d;
function spline(points: Vec2[], width?: number): Shape2d;
function arc(
  center: Vec2,
  radius: number,
  fromDegrees: number,
  toDegrees: number,
  width: number,
  options?: { caps?: "flat" | "round" },
): Shape2d;
function sector(
  center: Vec2,
  radius: number,
  fromDegrees: number,
  toDegrees: number,
): Shape2d;
function vesica(a: Vec2, b: Vec2, width: number): Shape2d;
function arch(min: Vec2, max: Vec2): Shape2d;
```

Angles in the plane run from +u toward +v.

1. `rect` spans its two corners in either order. `rect` takes `round` or
   `chamfer`. A chamfer cuts each corner at 45 degrees with legs that long.
2. `ngon` sets every side `radius` from its center with one side facing -v.
   `ngon(c, 4, r)` makes the square from `c - r` to `c + r`. An octagon's sides
   run along the axes and the diagonals.
3. `star` puts a point at +v.
4. `polygon` closes its outline back to the first point.
5. `polyline` and `arc` stroke a line `width` across for trim, filigree, and
   lettering. An arc's ends come round unless `caps` makes them flat.
6. `spline` draws a smooth curve through every point for bodies, arms, and
   scrolls. Without `width` the curve closes into an outline, and with `width`
   it runs open as a stroke.
7. `sector` cuts a pie slice from a disk.
8. `vesica` makes a pointed lens `width` across from `a` to `b` for leaves,
   blades, and flames.
9. `arch` tops the rectangle between its corners with a half circle as wide as
   the rectangle. `max` has to sit at least half the width above `min`. Doors,
   windows, and gravestones start from an `arch`.

### From 2D to 3D

```ts
function extrude(
  profile: Shape2d,
  options: { axis?: Axis; from: number; to: number },
): Shape3d;
function revolve(
  profile: Shape2d,
  options?: { axis?: Axis; center?: Vec3 },
): Shape3d;
function lathe(
  points: Vec2[],
  options?: { axis?: Axis; center?: Vec3 },
): Shape3d;
function sweep(
  profile: Shape2d,
  points: Vec3[],
  options?: { scales?: number[]; up?: Vec3 },
): Shape3d;
```

1. `extrude` pushes the profile from `from` to `to` along the z axis unless
   `axis` picks another. The profile's u and v map to the other two axes in x,
   y, z order: (x, y) for `"z"`, (x, z) for `"y"`, and (y, z) for `"x"`. Under
   `"y"` a profile's angles run from +x toward +z, opposite to a positive
   `rotate("y", ...)`.
2. `revolve` spins the profile around the y axis through `center` unless `axis`
   picks another. The profile's u measures the distance from the axis, and v
   measures the position along it. Only the half at u >= 0 counts.
3. `lathe` revolves the outline its `[radius, height]` points trace and closes
   the outline back along the axis. A `lathe` turns legs, balusters, vases, and
   finials.
4. `sweep` runs the profile along a smooth curve through every point for
   cables, tails, horns, and rails. The profile's origin rides the curve.
   Facing along the curve, v points as near `up` as the curve allows and u
   points right. `up` reads +y unless given. The curve may never run along
   `up`, so a rising trunk passes another `up`. `scales` sets the profile's
   scale at each point for tapers. The ends cut flat.

### Paths

```ts
function helix(
  base: Vec3,
  radius: number,
  height: number,
  turns: number,
  options?: { axis?: Axis; start?: number; perTurn?: number },
): Vec3[];
```

1. `helix` returns the points of a coil `radius` around the y axis through
   `base` unless `axis` picks another. The coil climbs `height` over `turns`
   turns with `perTurn` points per turn, which reads 12 unless given. `turns`
   has to give a whole number of points.
2. The first point sits `radius` along +z for the y axis, +x for z, and +y for
   x, turned `start` degrees the way `rotate` turns. Positive `turns` wind the
   way `rotate` turns, and negative ones wind back.

### Booleans

```ts
function union<S extends Shape3d | Shape2d>(...shapes: S[]): S;
function intersect<S extends Shape3d | Shape2d>(...shapes: S[]): S;
function subtract<S extends Shape3d | Shape2d>(base: S, ...cutters: S[]): S;
function smoothUnion<S extends Shape3d | Shape2d>(
  radius: number,
  ...shapes: S[]
): S;
function smoothIntersect<S extends Shape3d | Shape2d>(
  radius: number,
  ...shapes: S[]
): S;
function smoothSubtract<S extends Shape3d | Shape2d>(
  radius: number,
  base: S,
  ...cutters: S[]
): S;
```

The booleans work in 2D and 3D alike. A smooth union fills the inside corners
where its shapes meet with a fillet that reaches `radius` along each surface.
Shapes closer than `radius / 2` fuse. The other smooth variants round their
edges over the same reach.

Shapes carry no material. Geometry meets material only in a step, and one step's
shape can union many shapes under one material.

### Transforms

```ts
interface Shape3d {
  translate(offset: Vec3): Shape3d;
  rotate(axis: Axis, degrees: number, pivot?: Vec3): Shape3d;
  orient(from: Vec3, to: Vec3, pivot?: Vec3): Shape3d;
  scale(factor: number | Vec3, pivot?: Vec3): Shape3d;
  mirror(axes: Axes, center?: Vec3): Shape3d;
  repeat(step: Vec3, count: Vec3): Shape3d;
  repeatPolar(axis: Axis, count: number, center?: Vec3): Shape3d;
}
```

1. Transforms apply in call order and in world coordinates. `pivot` and `center`
   default to `[0, 0, 0]`.
2. `orient` turns the shape about `pivot` until the direction `from` points
   along `to`. A `torus` or a `lathe` can then point along any line.
3. `mirror` adds the shape's reflection across `center` along each axis in
   `axes`. Each letter flips one axis. `"x"` copies a left side to the right,
   and `leg.mirror("xz")` makes four legs from one. A shape crossing a mirror
   plane keeps both halves.
4. `repeat` makes `count` copies along each axis and offsets the copy at
   `[i, j, k]` by `[i * step[0], j * step[1], k * step[2]]`. The copy at
   `[0, 0, 0]` stays put. Five posts along z take
   `repeat([0, 0, s], [1, 1, 5])`.
5. `repeatPolar` makes `count` copies turned evenly around `axis` through
   `center`.

A `Shape2d` takes the same methods in the plane, plus the
[modifiers](#modifiers) `offset` and `shell`:

```ts
interface Shape2d {
  translate(offset: Vec2): Shape2d;
  rotate(degrees: number, pivot?: Vec2): Shape2d;
  scale(factor: number | Vec2, pivot?: Vec2): Shape2d;
  mirror(axes: "u" | "v" | "uv", center?: Vec2): Shape2d;
  repeat(step: Vec2, count: Vec2): Shape2d;
  repeatPolar(count: number, center?: Vec2): Shape2d;
  offset(distance: number): Shape2d;
  shell(thickness: number): Shape2d;
}
```

### Modifiers

```ts
interface Shape3d {
  offset(distance: number): Shape3d;
  shell(thickness: number): Shape3d;
  elongate(lengths: Vec3, center?: Vec3): Shape3d;
  twist(axis: Axis, degreesPerMeter: number, center?: Vec3): Shape3d;
  bend(along: Axis, toward: Side, radius: number, pivot?: Vec3): Shape3d;
  displace(options: {
    amplitude: number;
    scale: number;
    octaves?: number;
    seed: number;
  }): Shape3d;
}
```

1. `offset` grows the shape by `distance` and shrinks it when `distance` is
   negative. On an extrusion the offset moves the ends too. An inset rim takes
   the 2D `offset` before the `extrude`.
2. `shell` keeps the outer `thickness` and hollows the rest. A shell stays
   closed until a `carve` or a `subtract` cuts an opening.
3. `elongate` cuts the shape through `center` across each axis, moves the halves
   apart by `lengths`, and fills the gap with the cut's cross-section. A `torus`
   stretched along x makes a chain link. A pattern stretches with the shape.
4. `twist` turns each slice across `axis` by `degreesPerMeter` times the slice's
   distance from `center` along the axis.
5. `bend` curls the shape's `along` axis into an arc of `radius` curving toward
   `toward`. The slice through `pivot` stays put, and the arc keeps every length
   along the axis. A second `bend` keeps only its own pivot's slice in place and
   moves the rest of the first bend's arc.
6. `displace` roughens the surface with fractal noise up to `amplitude` deep
   whose features run about `scale` across. `octaves` sets how many layers the
   noise sums and defaults to 4.

`offset` and `shell` measure true thickness on:

1. Every primitive but `ellipsoid` and `boxFrame`
2. Extrusions of 2D primitives, and revolutions of 2D primitives that keep
   clear of the axis or are symmetric about it
3. An `intersect` or a `subtract` of such shapes
4. Such shapes moved, rotated, or scaled uniformly

A `union` or a `smoothUnion` measures each cell's depth to the nearest face of
any operand. That face can lie buried inside another operand. A `shell` or a
negative `offset` of a union therefore leaves walls or gaps where its operands
meet. A hollow form takes one shape without buried faces: a `box`, a `lathe`, or
an `extrude` of a `polygon`.

On an `ellipsoid`, a non-uniform scale, or an `elongate` along more than one
axis, the distance reads short. There a `shell` comes out thicker than asked,
and an `offset` moves farther. After `twist`, `bend`, or `displace`, the
distance can read long. A `shell` can then come out thinner and open holes.

### Pattern frames

A pattern reads each cell's position in the frame its step's shape was built in,
before the transform methods moved it. Grain built on an upright leg runs along
the leg after a `rotate` lays it flat. Every copy from `mirror` or `repeat`
shows the same pattern.

A primitive's arguments create no frame: a `cylinder` from `a` to `b` reads
world coordinates. Inside a union, a cell takes the frame of the operand it sits
deepest in, and the earlier operand wins a tie. Inside an `intersect` or a
`subtract`, a cell takes the frame of the first operand. `coat` and `set` read
world coordinates.

## Materials

```ts
const mat: Record<string, Material>;

function material(properties: Properties): Material;
function int(value: number | number[]): IntValue;
function json(value: unknown): JsonValue;
function shades(
  base: Material,
  options?: { count?: number; spread?: number },
): Material[];

interface Properties {
  baseColor: string;
  emissiveColor?: string;
  emissiveStrength?: number;
  ior?: number;
  metallic?: number;
  occlusionStrength?: number;
  roughness?: number;
  transmission?: number;
  [name: string]: Value | undefined;
}

type Value = boolean | number | string | number[] | IntValue | JsonValue;
```

1. `mat` holds the materials of the build's [libraries](#libraries) by name,
   such as `mat.oak` or `mat.ruby`.
2. `material` makes a material from the key/value properties a voxj palette
   holds. The eight listed properties follow voxj's
   [glTF conventions](https://github.com/tyleo/tyt/blob/main/projects/voxel-formats/voxj/docs/voxel-json-file-format.md#gltf-conventions)
   for their meanings, ranges, and defaults, except that `metallic` defaults
   to 0. The colors take sRGB hex: `#RRGGBB`, or `#RRGGBBAA` for a `baseColor`
   with alpha.
3. At `emissiveStrength` 1 a material glows at its `emissiveColor`. Strengths
   of 2 to 4 add a halo in the review renders and pale the core, and higher
   strengths whiten it. The lit `baseColor` adds to the glow. A dark
   `baseColor` keeps a strength-1 glow at its full hue.
4. Any other name adds a custom property whose kind follows its value: a
   boolean, a string, a float, or a vector of 2 to 4 floats. `int` marks whole
   numbers and vectors of them, and `json` carries any other JSON value. A
   material without a custom property that another material sets takes the
   kind's empty value there: 0, a zero vector, `false`, `""`, or `null`.
   `vxl palette show table.voxj --property '*' walkable value auto` lists the
   property `walkable` per material.
5. `shades` returns `count` versions of `base` that run from darkest to lightest
   with the original in the middle. Neighboring shades differ in perceived
   lightness by `spread`, and the other properties carry over. Darker shades mix
   the base with black. Lighter shades keep the base's hue and saturation.
   `count` defaults to 3 and `spread` to 0.05. A shade past black or white
   errors, and so does a lighter shade that sRGB cannot hold at the base's
   saturation. The error gives the largest `spread` that fits.

```ts
const glass = material({
  baseColor: "#D8F0FF",
  roughness: 0.05,
  transmission: 0.9,
});
const rune = material({
  baseColor: "#3A3F44",
  emissiveColor: "#40C0FF",
  emissiveStrength: 2,
});
const chest = material({
  baseColor: "#8A5A2B",
  lootTier: int(3),
  locked: true,
});
```

### The materials library

vxl defines the `materials` library, which names these materials:

| Group  | Names                                                                   |
| ------ | ----------------------------------------------------------------------- |
| Wood   | `bark`, `birch`, `ebony`, `mahogany`, `oak`, `pine`, `walnut`           |
| Stone  | `brick`, `clay`, `granite`, `marble`, `sandstone`, `slate`, `stone`     |
| Metal  | `brass`, `bronze`, `copper`, `gold`, `iron`, `silver`, `steel`          |
| Gem    | `amethyst`, `diamond`, `emerald`, `ruby`, `sapphire`, `topaz`           |
| Nature | `bone`, `dirt`, `grass`, `ice`, `leaf`, `moss`, `sand`, `snow`, `water` |
| Craft  | `leather`, `rope`, `straw`                                              |
| Light  | `ember`, `flame`, `glow`, `lava`                                        |

The metals are fully metallic, the gems are smooth and saturated, and the light
group glows. The gems, `ice`, and `water` also transmit light at their own
`ior`. Velvet, painted wood, and anything else the library lacks take a custom
`material`.

### Patterns

```ts
function bands(
  materials: Material[],
  options: { axis: Axis; period?: number; warp?: number; seed?: number },
): Pattern;
function grain(
  base: Material | Material[],
  options: { axis: Axis; period?: number; warp?: number; seed: number },
): Pattern;
function gradient(
  materials: Material[],
  options: {
    axis: Axis;
    from: number;
    to: number;
    warp?: number;
    seed?: number;
  },
): Pattern;
function noise(
  materials: Material[],
  options: { scale: number; octaves?: number; seed: number },
): Pattern;
function cells(
  materials: Material[],
  options: { size: number; seed: number; border?: Material },
): Pattern;
function speckle(
  base: Material,
  accents: Material | Material[],
  options: { density: number; seed: number },
): Pattern;
function checker(materials: Material[], options?: { size?: number }): Pattern;
```

A pattern picks one of its materials per cell and never blends them. The palette
then stays as small as the material lists. A seed always gives the same picks,
and a seed left out reads 0. A length a pattern leaves out counts cells at any
voxel size.

1. `bands` slices space across `axis` into slabs `period` thick and cycles
   through `materials` in order. `warp` bends the slabs by up to that much
   noise. `period` defaults to 1 cell and `warp` to 0.
2. `grain` cuts space into rings `period` thick around the line along `axis`
   through the frame's origin and gives each ring a random pick. Faces that run
   along `axis` streak, and end faces show the rings. A board, post, or leg
   takes the axis it runs along. A board built around the origin and moved by
   `translate` streaks across its faces. A board built in place far from the
   origin reads as one broad ring. `warp` bends the rings as it bends `bands`.
   `period` defaults to 2 cells and `warp` to 1.5 cells. A single `base` stands
   for `shades(base)`.
3. `gradient` splits `from` to `to` along `axis` into equal spans that take the
   materials in order. Cells before `from` take the first material and cells
   past `to` the last. `warp` bends the spans as it bends `bands` and defaults
   to 0.
4. `noise` splits fractal noise with features about `scale` across into value
   ranges that take the materials in order. Each material covers about an equal
   share of the cells. `octaves` defaults to 4.
5. `cells` breaks space into irregular cells about `size` across that each take
   a random pick. `border` fills a seam one grid cell wide between the cells for
   cobblestone and mortar. Cells under about 6 voxels across read mostly as
   border.
6. `speckle` gives each grid cell a `density` chance of a random accent and
   leaves the rest `base`. Because `speckle` reads grid cells, its copies
   differ.
7. `checker` alternates its materials over cubes `size` wide. `size` defaults to
   1 cell.

## Libraries

A library shares named entries between models. `lib` holds the entries of the
build's libraries by kind and name. `mat` holds `lib.materials`:

```ts
const lib: {
  shapes3d: Record<string, Shape3d>;
  shapes2d: Record<string, Shape2d>;
  materials: Record<string, Material>;
  patterns: Record<string, Pattern>;
  steps: Record<string, Step>;
  parts: Record<string, Part>;
};
```

```ts
export default [
  part("left", { offset: [-0.5, 0, 0] }, [lib.parts.stool]),
  part("right", { offset: [0.5, 0, 0] }, [lib.parts.stool]),
  add("counter", box([-1, 0.9, -0.3], [1, 0.95, 0.3]), mat.marble),
];
```

1. A later library wins a name
2. Reading a name no listed library holds errors
3. Using an entry copies the entry, every entry it references, and their names
   into the model's `.sdfj` document. The document then voxelizes without the
   library

Every built `.sdfj` document can serve as a library once a `.vxlconfig` gives it
a library name. A model file's named exports name their materials, patterns,
shapes, steps, and parts in the document. An exported function names nothing.
Any other named export errors. Exporting the array from `shades` errors, but
each shade can take its own export. A model that exports its own `oak` and also
uses `mat.oak` errors because two different materials take one name.

```ts
// woods.ts, built into woods.sdfj
export const walnut = material({ baseColor: "#5C4033", roughness: 0.6 });
export const stool = part("stool", {}, [
  add("seat", cylinder([0, 0.4, 0], [0, 0.44, 0], 0.2), walnut),
]);

export default [stool];
```

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
```

1. A step's cells are the cells it wrote as it ran: every cell an `add` or a
   `set` filled, every live cell a `carve` emptied, and every live cell a
   `paint` or a `coat` recolored.
2. The step keeps the cells no later step changed, and `lost to` lists the
   later steps that changed them. A `paint` or a `coat` line lists after `over`
   the steps whose cells it recolored.
3. `exposed` counts the kept cells that show: cells touching empty space across
   a face, and opaque cells touching a material that lets light through. A
   `carve` line has no `exposed` count.
4. A step whose cells form more than one face-connected piece gives the piece
   count. A cell a later step recolored still counts, and a cell a later step
   emptied does not.
5. A size counts cells along x, y, and z. Bounds run in meters from the min
   cell's min corner to the max cell's max corner and can feed a `box` directly.
   A step's size and bounds cover the cells it wrote.
6. A step with `0 cells` wrote nothing: an `add` or a `set` fell between the
   cell centers, or a `carve`, `paint`, or `coat` met no live cell. A step with
   cells and `0 kept` lost them all to the steps after `lost to`.
7. `--fill-mode surface` thins the voxel counts on the model, part, and piece
   lines to the shell the document holds. The other fields read the cells before
   the fill.

The legs read 4 pieces, one per leg. The top keeps 800 of its 968 cells because
the gilt takes the rest. The sapphires keep all 96 cells and show 48 of them
because half of each sapphire sits inside the apron.

With parts, each place of a part gets a part line with the part's pivot. The
part's steps and child parts follow, indented one level past the part line. A
part line counts the part's live cells. A part reads `detached` when neither it
nor a part below it shares a face-connected piece with its parent. A part in
several places lists its steps under its first place, and each later place's
line reads `repeats` and the first place's path. The model's line and the pieces
count the cells the parts cover together. A part reading `0 voxels` writes its
node with no object. The report for a model in more than one face-connected
piece ends with a line per piece. The piece lines run from the largest piece
down and list the steps whose live cells reach each piece. A step name that
repeats across lists takes its path of part names, such as `tree.1/tree/trunk`.

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
```

A second piece usually means a shape floats.

## Errors

Each command stops at its first error. A build error reports the failing call,
the argument, and the value. A voxelize error reports the failing value and the
failing step's path of part names. A step that writes no cell stops nothing and
reads `0 cells` in the report. The
[checks](https://github.com/tyleo/tyt/blob/main/doc/ref/sdf-doc/model-evaluation.md#checks)
list every error.

## Resolution

Voxel models run from 16 to several hundred voxels across. At any size the grid
shapes the smallest features:

1. A shape thinner than a voxel can miss every cell center and vanish. A feature
   one voxel thick holds only on whole cells along the axes: a one-voxel post is
   the `box` from `[x, y0, z]` to `[x + v, y1, z + v]`, with `x` and `z` on
   multiples of `v`. A tilted or curved member, a stroke, and a torus tube need
   about 2 voxels across or they break into pieces.
2. A sphere, cylinder, cone, octahedron, or rounded corner under a radius of
   about 3 voxels reads as a plus sign, a rod, or a block. Small round details
   read better as boxes or single `set` cells.
3. A large `add` placed after a detail buries it because later steps win. Large
   forms go first and details after. Nested shapes of one feature bury each
   other in any order.
4. Copies from `mirror` and `repeat` repeat their pattern exactly. A distinct
   look per copy takes its own step with its own seed.
5. A model holds its details at the voxel size they were sized for. A coarser
   sampling can drop details thinner than its cells, and a finer one draws each
   detail with more cells without adding any.
6. A thin form or a whole object turned off the axes samples into ribs and
   stair steps. Furniture, plates, and dishes read cleanest square to the axes.

## Recipes

Each recipe shows shapes as constants and steps as entries of the list, sized
for `--voxel-size 0.025`.

```ts
// Four turned legs from one profile.
const leg = lathe([[0.0375, 0], [0.03, 0.15], [0.045, 0.225], [0.03, 0.425]]).translate([0.2, 0, 0.2]);
add("legs", leg.mirror("xz"), grain(mat.oak, { axis: "y", seed: 1 })),

// A jewel set into a surface. The later add takes the cells the two share.
add("jewel", octahedron([0, 0.875, -0.175], 0.075), mat.ruby),

// A gilded top edge.
paint("gilt", intersect(rail.shell(0.025), halfSpace("+y", 0.925)), mat.gold),

// An open bowl.
add("bowl", subtract(sphere([0, 0.15, 0], 0.15).shell(0.025), halfSpace("+y", 0.2)), mat.clay),

// Twelve studs around a rim.
add("studs", box([0.225, 0.075, -0.025], [0.275, 0.125, 0.025]).repeatPolar("y", 12), mat.iron),

// Floor planks in alternating shades.
add("floor", box([-0.4, 0, -0.4], [0.4, 0.025, 0.4]), bands(shades(mat.pine), { axis: "x", period: 0.1 })),

// A cobbled wall with an arched doorway through it.
add("wall", box([0, 0, 0], [0.8, 0.4, 0.075]), cells(shades(mat.stone), { size: 0.15, seed: 7, border: mat.dirt })),
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
const walnut = (axis: Axis, seed: number) => grain(mat.walnut, { axis, seed });

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
