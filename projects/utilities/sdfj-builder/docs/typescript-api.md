# TypeScript API

A model file describes a voxel model in TypeScript as signed distance functions
measured in meters. Its code runs once and returns a list of steps. This section
lists every call a model file can make.

## Model files

A model file is a TypeScript module whose default export lists the model's steps
and parts. Every name in this section sits in scope, and a model imports none of
them:

```ts
export default [add("block", box([0, 0, 0], [0.1, 0.1, 0.1]), mat.stone)];
```

Variables, functions, and loops shape the list. Nothing in a model runs per
voxel because shapes, steps, and patterns only describe what gets sampled later.

A model holds no voxel size.

A variable named like a call in this section hides the call. A material held in
`const paint` breaks every `paint` step.

`vars` holds the build variables as JSON values by name. Reading a variable the
build does not set errors:

```ts
// Built with legs set to 3.
const leg = (i: number) =>
  add(`leg${i}`, cylinder([i * 0.1, 0, 0], [i * 0.1, 0.4, 0], 0.02), mat.oak);

export default Array.from({ length: vars.legs }, (_, i) => leg(i));
```

A larger model can split across files. A file imports another by its relative
path with the `.ts` extension. Every file sees the names in this section.

A named export gives its value a name in the built document, which can then
serve as a [library](#library-exports).

## Coordinates

The axes follow glTF:

| Quantity                    | Convention                  |
| --------------------------- | --------------------------- |
| Up                          | +Y                          |
| Right                       | +X                          |
| Toward the viewer           | +Z                          |
| The model's front           | Faces +Z                    |
| Positions, distances, radii | Meters                      |
| Angles                      | Degrees                     |
| A positive rotation         | Follows the right-hand rule |

Under the right-hand rule, a positive rotation turns counterclockwise when seen
from the positive end of its axis.

## Types

```ts
type Vec2 = [number, number];
type Vec3 = [number, number, number];
type Axis = "x" | "y" | "z";
type Axes = "x" | "y" | "z" | "xy" | "xz" | "yz" | "xyz";
type Side = "+x" | "-x" | "+y" | "-y" | "+z" | "-z";
type AxisRotation = [Axis, number];
```

| Type       | Holds                                                   |
| ---------- | ------------------------------------------------------- |
| `Shape3d`  | A 3D region                                             |
| `Shape2d`  | A 2D region in a plane with axes u and v                |
| `Material` | Surface properties                                      |
| `Pattern`  | A pick of one material per cell                         |
| `Step`     | One entry of a part's list                              |
| `Part`     | Steps voxelized into an object that turns about a joint |

These values never change. Because a method returns a new value and leaves its
receiver alone, one shape can feed several steps.

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
  size: number,
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

A part's steps apply in list order to the part's grid. A later step wins
wherever two steps reach the same cell.

| Step    | Effect                                                                                                                         |
| ------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `add`   | Fills the shape's cells with the material and replaces what they held                                                          |
| `carve` | Empties the shape's cells                                                                                                      |
| `paint` | Recolors the live cells inside the shape and fills none                                                                        |
| `coat`  | Recolors a live cell when the point `depth` meters from its center toward a listed side lies outside the part's earlier shapes |
| `set`   | Fills the cells whose centers lie in a cube `size` wide around each point                                                      |

`coat` takes these settings:

1. `depth` takes at least one voxel, and `depth: v` coats the outer layer of
   cells
2. `sides` defaults to all six
3. `within` limits the coat to a shape's cells

`set` places detail too small for a shape. A `size` of `v` around a cell center,
`(i + 0.5) * v`, fills that one cell.

`onto` keeps a `paint` or a `coat` on the cells the listed `add` steps own:

1. A cell belongs to the last `add` that filled it until a `carve` empties it
2. Each step in `onto` has to be an `add` earlier in the same list

Every step takes a name no other step in its list shares. The report and errors
use the name.

A part's grid grows to hold every `add` and `set`. `carve`, `paint`, and `coat`
never grow it.

## Parts

```ts
function part(
  name: string,
  options: {
    pivot?: Vec3;
    offset?: Vec3;
    rotation?: AxisRotation[];
    joint?: boolean;
  },
  list: (Step | Part)[],
): Part;
```

A part groups steps into a voxel object that turns about `pivot`. A part in
another part's list becomes that part's child.

Arms, wings, hands, wheels, and lids take parts when the prompt asks for pieces
that move or separate objects.

The options place and pose the part:

1. `pivot` defaults to `[0, 0, 0]`
2. `offset` moves the part's steps, child parts, and pivot. The offsets of
   nested parts add up
3. `rotation` poses the part. Each `[axis, degrees]` in list order rotates the
   part about its pivot on an axis of the parent's frame
4. `joint: true` marks a part that an animation turns, such as a lid or a door.
   Flattening the parts keeps each joint's node

A posed part voxelizes unrotated, and its node takes the rotation.

Parts voxelize this way:

1. A part's steps run in list order over the part's grid
2. A child part voxelizes on its own grid, and parts can overlap in space
3. `carve`, `paint`, and `coat` reach only the cells of their part, which keeps
   each part whole when a joint turns
4. Each part writes its object under a node at its pivot, and the nodes nest as
   the parts do
5. Turning a part's node turns the part and its children about the joint
6. A step or a part can sit in several lists. A part in several lists writes one
   node that every parent's node lists

The steps and parts outside every part belong to the root part. The root part
takes the model file's stem as its name and turns about `[0, 0, 0]`.

Every part takes a name no other part in its list shares.

A function builds symmetric parts:

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

1. `box` spans its two corners in either order on each axis. `round` rounds its
   edges and corners by that radius inside the same corners
2. `boxFrame` keeps the twelve edges of the box between its corners as square
   bars `thickness` across. The bars stay inside the corners. Crates, cages, bed
   frames, and lantern frames start from a `boxFrame`
3. `cylinder`, `cone`, `capsule`, and `roundCone` run from `a` to `b` at any
   angle
4. `cylinder` takes `round` to round its rims
5. `cone` takes `radiusA` at `a` and `radiusB` at `b`. A radius of 0 makes a
   point
6. `roundCone` caps each end with a sphere of that end's radius and tapers
   between the spheres. Limbs, branches, horns, and bottle necks suit a
   `roundCone`
7. `torus` rings the y axis unless `axis` picks another. `ringRadius` reaches
   from the center to the middle of the tube, and `tubeRadius` sets the tube's
   radius
8. `torus` takes `from` and `to` to cut the ring to an arc with round ends. The
   angles run as an `arc`'s do in the plane `extrude` maps across `axis`. A cut
   torus makes handles, hooks, and horseshoes
9. `octahedron` reaches `radius` along each axis from its center. From a radius
   of about 3 voxels it makes a cut-gem silhouette. A smaller one reads as a
   plus sign
10. `pyramid` stands on a square base `width` across centered on `baseCenter`.
    Its apex sits `height` above the base along +Y
11. `halfSpace` covers everything past `at` toward `side`.
    `halfSpace("+y", 0.25)` holds every point above `y = 0.25`

A half space has no bounds. An `add` takes a half space only inside an
`intersect` or as a cutter. `carve`, `paint`, and `coat` take one bare.

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

1. `rect` spans its two corners in either order and takes `round` or `chamfer`.
   A chamfer cuts each corner at 45 degrees with legs that long
2. `ngon` sets every side `radius` from its center with one side facing -v.
   `ngon(c, 4, r)` makes the square from `c - r` to `c + r`. An octagon's sides
   run along the axes and the diagonals
3. `star` puts a point at +v
4. `polygon` closes its outline back to the first point
5. `polyline` and `arc` stroke a line `width` across for trim, filigree, and
   lettering. An arc's ends come round unless `caps` makes them flat
6. `spline` draws a smooth curve through every point for bodies, arms, and
   scrolls. Without `width` the curve closes into an outline. With `width` it
   runs open as a stroke
7. `sector` cuts a pie slice from a disk
8. `vesica` makes a pointed lens `width` across from `a` to `b` for leaves,
   blades, and flames
9. `arch` tops the rectangle between its corners with a half circle as wide as
   the rectangle. `max` has to sit at least half the width above `min`. Doors,
   windows, and gravestones start from an `arch`

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

`extrude` pushes the profile from `from` to `to` along the z axis unless `axis`
picks another. The profile's u and v map to the other two axes in x, y, z order:

| `axis` | u, v map to |
| ------ | ----------- |
| `"x"`  | y, z        |
| `"y"`  | x, z        |
| `"z"`  | x, y        |

Under `"y"`, a profile's angles run from +x toward +z, opposite to a positive
`rotate("y", ...)`.

`revolve` spins the profile around the y axis through `center` unless `axis`
picks another. The profile's u measures the distance from the axis, and v
measures the position along it. Only the half at u >= 0 counts.

`lathe` revolves the outline its `[radius, height]` points trace and closes the
outline back along the axis. A `lathe` turns legs, balusters, vases, and
finials.

`sweep` runs the profile along a smooth curve through every point for cables,
tails, horns, and rails:

1. The profile's origin rides the curve
2. Facing along the curve, v points as near `up` as the curve allows and u
   points right
3. `up` defaults to +y
4. The curve may never run along `up`, so a rising trunk passes another `up`
5. `scales` sets the profile's scale at each point for tapers
6. The ends cut flat

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

`helix` returns the points of a coil `radius` around the y axis through `base`
unless `axis` picks another:

1. The coil climbs `height` over `turns` turns
2. `perTurn` sets the points per turn and defaults to 12
3. `turns` has to give a whole number of points
4. Positive `turns` wind the way `rotate` turns, and negative ones wind back

The first point sits `radius` from the axis, turned `start` degrees the way
`rotate` turns from this direction:

| `axis` | First point before `start` |
| ------ | -------------------------- |
| `"x"`  | Along +y                   |
| `"y"`  | Along +z                   |
| `"z"`  | Along +x                   |

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

The booleans work in 2D and 3D alike.

`smoothUnion` fills the inside corners where its shapes meet with a fillet that
reaches `radius` along each surface. Shapes closer than `radius / 2` fuse.

`smoothIntersect` and `smoothSubtract` round their edges over the same reach.

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

Transforms apply in call order and in world coordinates. `pivot` and `center`
default to `[0, 0, 0]`.

1. `orient` turns the shape about `pivot` until the direction `from` points
   along `to`. A `torus` or a `lathe` can then point along any line
2. `scale` multiplies the shape's offset from `pivot` by `factor` on each axis
3. A negative `scale` factor reflects that axis and keeps only the reflection.
   `scale([-1, 1, 1])` turns a left side into a right one
4. `mirror` adds the shape's reflection across `center` along each axis in
   `axes`, one axis per letter. `"x"` copies a left side to the right, and
   `leg.mirror("xz")` makes four legs from one
5. A shape crossing a mirror plane keeps both halves. A reflection alone takes a
   negative `scale`
6. `repeat` makes `count` copies along each axis. The copy at `[i, j, k]` moves
   by `[i * step[0], j * step[1], k * step[2]]`, and the copy at `[0, 0, 0]`
   stays put. Five posts along z take `repeat([0, 0, s], [1, 1, 5])`
7. `repeatPolar` makes `count` copies turned evenly around `axis` through
   `center`

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

1. `offset` grows the shape by `distance`. A negative `distance` shrinks it
2. `offset` on an extrusion moves the ends too. An inset rim takes the 2D
   `offset` before the `extrude`
3. `shell` keeps the outer `thickness` and hollows the rest. A shell stays
   closed until a `carve` or a `subtract` cuts an opening
4. `elongate` cuts the shape through `center` across each axis, moves the halves
   apart by `lengths`, and fills the gap with the cut's cross-section. A pattern
   stretches with the shape. A `torus` stretched along x makes a chain link
5. `twist` turns each slice across `axis` by `degreesPerMeter` times the slice's
   distance from `center` along the axis
6. `bend` curls the shape's `along` axis into an arc of `radius` curving toward
   `toward`. The slice through `pivot` stays put, and the arc keeps every length
   along the axis
7. A second `bend` keeps only its own pivot's slice in place and moves the rest
   of the first bend's arc
8. `displace` roughens the surface with fractal noise up to `amplitude` deep
   whose features run about `scale` across. `octaves` sets how many layers the
   noise sums and defaults to 4

#### True thickness

`offset` and `shell` measure true thickness on:

1. Every primitive but `ellipsoid` and `boxFrame`
2. Extrusions of 2D primitives, and revolutions of 2D primitives that keep clear
   of the axis or are symmetric about it
3. An `intersect` or a `subtract` of such shapes
4. Such shapes moved, rotated, or scaled uniformly

A `union` or a `smoothUnion` measures each cell's depth to the nearest face of
any operand. That face can lie buried inside another operand. A `shell` or a
negative `offset` of a union then leaves walls or gaps where its operands meet.

A hollow form takes one shape without buried faces: a `box`, a `lathe`, or an
`extrude` of a `polygon`.

On an `ellipsoid`, a non-uniform scale, or an `elongate` along more than one
axis, the distance reads short. A `shell` there comes out thicker than asked,
and an `offset` moves farther.

After `twist`, `bend`, or `displace`, the distance can read long. A `shell` can
then come out thinner and open holes.

### Pattern frames

A pattern reads each cell's position in the frame its step's shape was built in,
before the transform methods moved it. Grain built on an upright leg runs along
the leg after a `rotate` lays it flat. Every copy from `mirror` or `repeat`
shows the same pattern.

| Where the cell sits            | Frame the cell reads                                                 |
| ------------------------------ | -------------------------------------------------------------------- |
| A primitive                    | World coordinates                                                    |
| A union                        | The operand the cell sits deepest in. The earlier operand wins a tie |
| An `intersect` or a `subtract` | The first operand                                                    |
| A `coat` or a `set` step       | World coordinates                                                    |

A primitive's arguments create no frame: a `cylinder` from `a` to `b` reads
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

`mat` holds the materials of the build's [libraries](#libraries) by name, such
as `mat.oak` or `mat.ruby`.

`material` makes a material from the key/value properties a voxj palette holds.
The eight listed properties follow voxj's
[glTF conventions](https://github.com/tyleo/tyt/blob/main/projects/voxel-formats/voxj/docs/voxel-json-file-format.md#gltf-conventions)
for their meanings, ranges, and defaults. Only `metallic` departs from them, by
defaulting to 0.

The colors take sRGB hex: `#RRGGBB`, or `#RRGGBBAA` for a `baseColor` with
alpha.

### Custom properties

Any other name adds a custom property whose kind follows its value:

| Value                              | Kind                           |
| ---------------------------------- | ------------------------------ |
| A boolean                          | Boolean                        |
| A string                           | String                         |
| A number                           | Float                          |
| An array of 2 to 4 numbers         | Vector of floats               |
| `int(...)` of a number or an array | Whole number or vector of them |
| `json(...)`                        | Any other JSON value           |

A material without a custom property that another material sets takes the kind's
empty value there: 0, a zero vector, `false`, `""`, or `null`.

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

### Shades

`shades` returns `count` versions of `base` that run from darkest to lightest
with the original in the middle:

1. `count` defaults to 3
2. Neighboring shades differ in perceived lightness by `spread`, which defaults
   to 0.05
3. The other properties carry over
4. Darker shades mix the base with black
5. Lighter shades keep the base's hue and saturation

A shade past black or white errors. So does a lighter shade that sRGB cannot
hold at the base's saturation. The error gives the largest `spread` that fits.

### Patterns

```ts
function bands(
  materials: Material[],
  period: number,
  options: { axis: Axis; warp?: number; seed?: number },
): Pattern;
function grain(
  base: Material | Material[],
  period: number,
  options: { axis: Axis; warp?: number; seed: number },
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
  scale: number,
  options: { octaves?: number; seed: number },
): Pattern;
function cells(
  materials: Material[],
  size: number,
  options: { seed: number; border?: { material: Material; width: number } },
): Pattern;
function speckle(
  base: Material,
  accents: Material | Material[],
  size: number,
  options: { density: number; seed: number },
): Pattern;
function checker(materials: Material[], size: number): Pattern;
```

A pattern picks one of its materials per cell and never blends them. The palette
then stays as small as the material lists.

A seed always gives the same picks. A seed left out reads 0.

1. `bands` slices space across `axis` into slabs `period` thick and cycles
   through `materials` in order
2. `grain` cuts space into rings `period` thick around the line along `axis`
   through the frame's origin and gives each ring a random pick. A single `base`
   stands for `shades(base)`
3. `gradient` splits `from` to `to` along `axis` into equal spans that take the
   materials in order. Cells before `from` take the first material, and cells
   past `to` take the last
4. `noise` splits fractal noise with features about `scale` across into value
   ranges that take the materials in order. Each material covers about an equal
   share of the cells. `octaves` defaults to 4
5. `cells` breaks space into irregular cells about `size` across that each take
   a random pick. `border` fills a seam `width` wide between the cells with its
   `material` for cobblestone and mortar. A `width` under the voxel size leaves
   gaps in the seam
6. `speckle` gives each cube `size` wide on the grid a `density` chance of a
   random accent and leaves the rest `base`. Because `speckle` reads cubes fixed
   to the grid, its copies differ. A `size` of `v` accents single cells
7. `checker` alternates its materials over cubes `size` wide

`warp` bends a pattern's divisions by up to that much noise:

| Pattern    | `warp` bends | `warp` default   |
| ---------- | ------------ | ---------------- |
| `bands`    | The slabs    | 0                |
| `grain`    | The rings    | 0.75 of `period` |
| `gradient` | The spans    | 0                |

On a `grain`, faces that run along `axis` streak, and end faces show the rings:

1. A board, post, or leg takes the axis it runs along
2. A board built around the origin and moved by `translate` streaks across its
   faces
3. A board built in place far from the origin reads as one broad ring

## Libraries

A library shares named entries between models. `lib` holds the entries of the
build's libraries by kind and name, and `mat` holds `lib.materials`:

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
   into the model's built document. The document then voxelizes without the
   library

### Library exports

A model file's named exports name their materials, patterns, shapes, steps, and
parts in the built document. The document can then serve as a library.

1. An exported function names nothing
2. Any other named export errors
3. Exporting the array from `shades` errors, but each shade can take its own
   export
4. A model that exports an `oak` and also uses `mat.oak` errors because two
   different materials take one name

```ts
// woods.ts
export const walnut = material({ baseColor: "#5C4033", roughness: 0.6 });
export const stool = part("stool", {}, [
  add("seat", cylinder([0, 0.4, 0], [0, 0.44, 0], 0.2), walnut),
]);

export default [stool];
```
