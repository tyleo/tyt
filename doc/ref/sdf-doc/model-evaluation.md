# Model evaluation

_Part of the [voxel modeling plan](../../plan/open/voxel-modeling/README.md)._

This page sets what vxl computes from a model. The
[modeling API](../../../projects/utilities/vxl/docs/modeling-api.md) lists the
calls. Most formulas come from Inigo Quilez's MIT-licensed
[articles](https://iquilezles.org/articles/). Each section links the articles it
follows.

## Pipeline

`vxl sdf-doc build` turns a model file into an `.sdfj` document:

1. The flags and profile merge into the [settings](#settings).
2. vxl reads the [libraries](#libraries) the settings list and writes them with
   the builder's files to a temporary directory. vxl then runs the builder on
   the model under the runtime. The runtime shares vxl's standard output and
   error.
3. The builder imports the model with the API in scope. The default export has
   to be an array of steps and parts.
4. The builder writes the calls as the [`.sdfj` document](#the-sdfj-document)
   in the temporary directory. vxl copies the document to the output path when
   the runtime exits with code 0. Any other exit fails the build and leaves the
   output path untouched.

`vxl sdf-doc voxelize` turns the `.sdfj` document into voxj:

1. The flags and profile merge into the [settings](#settings).
2. vxl reads the document and runs the [checks](#checks) over it.
3. The [sampling](#sampling) settles the voxel size.
4. Each part's [grid](#the-grid) wraps the part's `add` shapes and `set` points.
5. Each part's [steps](#steps) run in list order over its grid. The fill mode
   follows the last step.
6. The [palette](#materials) collects the materials the final cells hold. vxl
   writes the [voxj document](#the-voxj-document). `--report` prints the
   [report](#the-report) and writes it beside the document as
   `<stem>-report.txt`.

## Settings

Both commands merge their flags and profile as
[`vxl mesh-doc voxelize`](../../plan/open/vxl-commands/reference/mesh-doc/voxelize.md)
does.

1. `--profile` reads `sdfDoc.build.profiles` or `sdfDoc.voxelize.profiles`
   through the `.vxlconfig` cascade that
   [`mesh-doc voxelize` profiles](../../plan/open/vxl-commands/reference/mesh-doc/voxelize.md#profiles)
   load. The cascade holds no built-in profiles. A name reads from the last file
   supplying it.
2. A profile holds the flags by camel-case name with their command-line values.
   A build profile holds `runtime` and `libraries` as an array of library
   names. A voxelize profile holds `resolution` as an
   object of `reference` and `count`, `voxelSize`, `frame`, `fillMode`,
   `flatten`, and `report`. A voxelize profile sets at most one of `resolution`
   and `voxelSize`. Either profile can also hold a one-line `description`. An
   unknown key or value errors when the profiles load.
3. A flag on the command line replaces the value the profile sets. Either
   `--resolution` or `--voxel-size` replaces the profile's `resolution` and
   `voxelSize` alike.
4. The output paths and the voxelize encoding flags stay on the command line.
   The encoding flags default as they do for `mesh-doc voxelize`.

## The `.sdfj` document

The builder records the model and computes nothing. The
[sdfj format](../../../projects/sdf-formats/sdfj/docs/sdf-json-file-format.md)
sets the document. vxl expands and checks every value when it voxelizes.

## Libraries

1. `sdfDoc.build.libraries` defines libraries by name through the `.vxlconfig`
   cascade, with vxl's built-in layer first. `files` maps a name to an optional
   `description` and a `path`, which resolves against the directory of the
   `.vxlconfig` holding it. `embedded` maps a name to an optional `description`
   and an `.sdfj` `document`.
2. A name reads from the last layer defining it in either group. One layer
   defining a name in both groups errors.
3. vxl's built-in layer embeds `materials`, which names a material for each name
   the modeling API lists.
4. vxl reads each library the settings list and checks it by the
   [sdfj format](../../../projects/sdf-formats/sdfj/docs/sdf-json-file-format.md#rules).
   The builder reads the libraries in list order, and a later library wins a
   name.
5. `lib` and `mat` read an entry the first time the model uses its name. One
   library entry gives one value however often the model uses the entry.
   Reading an entry also reads every entry it references.
6. A library part reads from its node. The node's object gives the part's steps
   in order, and the node's children give its child parts.
7. The builder writes a library value as it writes any value. A written entry
   takes each name it holds in its library and the name of each named export
   holding it.

## Sampling

Each shape is a signed distance function `d(p)` in meters that reads negative
inside. The sampling sets the voxel size `g` from the settings.

1. `--voxel-size` sets `g`. Without a size from the command line or the profile,
   `g` reads 1.
2. `--resolution <reference> <n>` sets `g` to the reference side divided by `n`.
   The sides come from boxes taken before any rounding.
3. World references measure the box around every part's `add` shapes and `set`
   points at every place.
4. Object references measure each part's box and take the extreme across parts,
   with or without `--flatten`. A part's box wraps the `add` shapes and `set`
   points in the part's list. A part with none has no box.

The lattice anchors at the origin. The cell `[i, j, k]` spans from
`[i, j, k] * g` to `[i + 1, j + 1, k + 1] * g`. vxl samples each part's shapes
moved by a shift `o`. Under `--frame world`, `o` adds up the offsets on a
place's path, and vxl samples each place separately. Under `--frame local`, `o`
reads zero, and vxl samples a part once for all its places. A cell belongs to a
shape when `d` at the point `[i + 0.5, j + 0.5, k + 0.5] * g - o` reads zero or
less, and a point exactly on the surface counts as inside. Only the sign decides
membership. A shape whose `d` bounds the true distance therefore voxelizes
exactly as an exact one does.

## Arithmetic

All arithmetic runs in f64. Addition, subtraction, multiplication, division, and
the square root round correctly on every platform. A platform's sine, cosine,
and arc tangent can differ in the last bit. The rules below keep trigonometry
off the path of most cells.

1. vxl computes these sines and cosines once as it reads the document:
   - rotations and `repeatPolar` copies
   - the directions of `ngon` and `star`
   - the ends of `arc`, `sector`, and a cut `torus`
   - the middle of a `bend`

   An angle first reduces into `[0, 360)` degrees. Multiples of 90 give exact
   zeros and ones, and the other multiples of 30 and 45 take `1 / 2`,
   `sqrt(2) / 2`, and `sqrt(3) / 2`. A quarter turn then keeps a box's corners
   on the cell corners.

2. `twist` takes a sine and a cosine per point, and `bend` takes an arc tangent.
   A cell whose center lies within rounding of a twisted or bent surface may
   land either way between platforms.
3. A length sums its squares in coordinate order and takes the square root. A
   library `hypot` never stands in.
4. No formula calls a sign function or a library rounding function. Each sign
   comes from a comparison, and the formula states which side zero falls on.
   Rounding to the nearest whole number computes `floor(x + 0.5)`.
5. The [checks](#checks) reject every degenerate input. A NaN that still appears
   stops the run as a bug.

## Distances

### Primitives

The [3D](https://iquilezles.org/articles/distfunctions/) and
[2D](https://iquilezles.org/articles/distfunctions2d/) catalogs supply the
formulas the lines below cite. Every primitive gives an exact signed distance
unless its line says otherwise.

1. `sphere` gives `length(p - center) - radius`.
2. With `h` its half extents, `box` folds the point about its center into
   `q = abs(p - center) - h` and gives `length(max(q, 0)) + min(max(q), 0)`.
   `round` shrinks `h` by the radius and subtracts the radius.
3. `boxFrame` takes the least box distance among its three bar directions after
   folding the point into one octant. The frame reads short inside where its
   bars meet.
4. `cylinder`, `cone`, `capsule`, and `roundCone` measure in the half-plane
   through their axis. The point's radial distance `k` takes the length of its
   offset from the axis line and never comes from a difference of squares. `v`
   gives the point's position along the axis.
   - A `cylinder` gives the box distance of `[k, v]` with half extents
     `[radius, length / 2]` and `v` measured from the midpoint. `round` shrinks
     and subtracts the same way.
   - A `cone` measures to the trapezoid between its end radii by the capped cone
     formula.
   - A `capsule` measures to its segment and subtracts the radius.
   - A `roundCone` measures to its two end circles and their outer tangents by
     the round cone formula.
5. `torus` gives `length([k - ringRadius, v]) - tubeRadius` about its axis. A
   cut torus turns the plane so the arc's middle lies on +v and folds u to
   `abs(u)`. A point past the arc's end then measures to the ring's end point.
6. `octahedron` uses the exact octahedron formula and not the bound.
7. `pyramid` evaluates the unit-base pyramid at `(p - baseCenter) / width` with
   height `height / width` and multiplies the result by `width`. The pyramid
   formula measures to the slanted faces. At or below the base plane the
   distance runs to the base square instead, and inside it takes the nearer of
   the faces and the base.
8. `ellipsoid` gives a bound from the
   [ellipsoid article](https://iquilezles.org/articles/ellipsoids/). With
   `q = p - center`, `k0 = length(q / radii)`, and
   `k1 = length(q / (radii * radii))`, it reads `k0 * (k0 - 1) / k1` where
   `k0 >= 1` and `(k0 - 1) * min(radii)` elsewhere. The first stays tight
   outside, and the second never reads deeper than the true depth or divides by
   zero at the center.
9. `halfSpace` gives the signed distance to its plane.

In the plane:

1. `circle` gives `length(p - center) - radius`.
2. `rect` gives the box distance in the plane. `round` shrinks and subtracts the
   same way, and `chamfer` uses the chamfer box formula.
3. `ellipse` runs the trig-free Newton iteration from the
   [ellipse article](https://iquilezles.org/articles/ellipsedist/) exactly 10
   times from its starting guess. The iteration refines the nearest point's
   cosine and sine, and the distance runs to the point it lands on. With radii
   `a` and `b`, the sign comes from the division-free test
   `b * b * x * x + a * a * y * y <= a * a * b * b`. Equal radii take the
   circle's formula.
4. `ngon` and `star` fold the point into one sector. Each builds its directions
   once: the side midpoints of an `ngon` and the tips of a `star`. A point turns
   by the rotation that takes its direction of largest dot product onto +v, and
   the earliest direction wins a tie. The point then folds u to `abs(u)`.
   - An `ngon` measures to its one side at `v = radius` and reads inside where
     `v <= radius`.
   - A `star` measures to the edge from its tip at `[0, outerRadius]` to the
     inner vertex at `innerRadius` halfway to the next tip. It reads inside on
     the center's side of that edge.
5. `polygon` measures to its nearest edge through the least squared edge
   distance and one square root. Its sign follows the even-odd rule along +u.
   With `e = v[j] - v[i]` for the edge from `v[i]` back to the previous vertex
   `v[j]` and `w = p - v[i]`, the edge flips the sign when the tests
   `p.v >= v[i].v`, `p.v < v[j].v`, and `e.u * w.v > e.v * w.u` all hold or all
   fail.
6. `polyline` measures to its nearest segment the same way and subtracts half
   the width.
7. `arc` turns the plane so the arc's middle lies on +v and folds u to `abs(u)`.
   With `s` and `c` the sine and cosine of half its span, a point where
   `c * u > s * v` measures to the arc's end point, and any other point to the
   circle as `abs(length(p) - radius)`. Both then subtract half the width. Flat
   caps instead cut the band square at the ends by the ring formula.
8. `sector` folds the same way and takes the larger of the circle's distance and
   the signed distance to the end's radius segment by the pie formula.
9. `vesica` uses the oriented vesica formula with half its width.
10. `arch` measures to its outline of base, sides, and half circle by the tunnel
    formula.

### From 2D to 3D

An exact profile gives an exact extrusion. A revolution stays exact when its
profile keeps clear of the axis or is symmetric about it, and reads short near
the axis otherwise.

1. `extrude` combines the profile's distance `d2` with the distance `h` to the
   slab between `from` and `to` as
   `min(max(d2, h), 0) + length([max(d2, 0), max(h, 0)])`.
2. `revolve` evaluates the profile at `[k, v]`, with `k` the point's distance
   from the axis and `v` its position along the axis from `center`.
3. `lathe` revolves the `polygon` of its points followed by their mirror images
   across the axis in reverse order. The outline closes across the axis at its
   first and last heights, and no edge runs along the axis.

### Booleans

The smooth formulas follow the quadratic
[smooth minimum](https://iquilezles.org/articles/smin/) with the article's `k`
equal to `r / 4`. With operand distances `a` and `b` and the radius `r`:

| Boolean           | Distance                                                          |
| ----------------- | ----------------------------------------------------------------- |
| `union`           | `min(a, b)`                                                       |
| `intersect`       | `max(a, b)`                                                       |
| `subtract`        | `max(a, -b)`                                                      |
| `smoothUnion`     | `min(a, b) - h * h * r / 4` with `h = max(r - abs(a - b), 0) / r` |
| `smoothIntersect` | `max(a, b) + h * h * r / 4` with the same `h`                     |
| `smoothSubtract`  | `smoothIntersect` of `a` and `-b`                                 |

More than two operands fold left. The smooth folds depend on the order.

A `union` is exact outside and reads short inside. Inside, a `union` measures to
the nearest operand's face even when another operand buries that face.
[Interior distances](https://iquilezles.org/articles/interiordistance/) covers
the effect. An `intersect` and a `subtract` are exact inside and read short
outside. On exact operands the smooth variants never read long and read exact
where `abs(a - b) >= r`.

### Transforms

A transform evaluates its child at the point mapped back through its inverse.
Translation, rotation, and `orient` keep the distance exact. A uniform scale `s`
returns `s * d(p / s)`. A non-uniform scale returns `min(s) * d(p / s)` and
reads short.

`orient` normalizes its two directions once and builds its rotation from them
with the trig-free alignment from
[avoiding trigonometry](https://iquilezles.org/articles/noacos/).

`mirror`, `repeat`, and `repeatPolar` return the union of their copies: the
minimum of `d` over the point mapped back into the original by each reflection,
translation, or rotation. Each copy's map builds once. `repeatPolar` turns the
point by each copy's precomputed rotation rather than folding the point by its
angle. An implementation may skip a copy whose box distance exceeds the running
minimum. The [box clamp](#bounds) keeps every copy's distance at or above its
box distance. A skipped copy therefore never changes the minimum.

### Modifiers

| Modifier    | Distance                         |
| ----------- | -------------------------------- |
| `offset(r)` | `d - r`                          |
| `shell(t)`  | `max(d, -d - t)`                 |
| `displace`  | `d + amplitude * fbm(p / scale)` |

`shell(t)` keeps the cells with `-t <= d <= 0`. On an exact distance, a `t` of
one voxel keeps a layer one cell deep.

`elongate` evaluates its child at `p - clamp(p - center, -h, h)` with `h` half
its lengths. An `elongate` along one axis stays exact, and one along more axes
reads short inside.

`twist` turns the point about the axis line through `center` by
`-degreesPerMeter` times its offset along the axis from `center` and then
evaluates the child.

`bend` maps each point back to the unbent shape:

1. The arc's center `C` sits `radius` from `pivot` toward `toward`.
2. The middle of the child's box along `along`, before the box rounds to the
   lattice, bends to the direction `m` from `C`. The angle from the direction of `pivot` to `m` is
   `phiM = (middle - pivot) / radius` and runs positive toward +`along`.
3. With `u` the point's offset from `C` in the bend plane, the point's angle is
   `phiM` plus the signed angle from `m` to `u`. The signed angle takes the
   `atan2` of the cross and dot products of `m` and `u`.
4. The point's angle times `radius` gives its unbent offset from `pivot` along
   `along`, and `radius - length(u)` gives its unbent offset from `pivot` toward
   `toward`. The third coordinate stays.

Measuring from `m` keeps the arc tangent's branch cut opposite the bent shape.

`twist`, `bend`, and `displace` can read long where they squeeze space. A
`shell` after any of the three may open holes. `offset` and `shell` measure true
thickness only on an exact input.

### Noise

`fbm(q)` sums `octaves` layers of
[gradient noise](https://iquilezles.org/articles/gradientnoise/) and maps the
sum into `(-1, 1)`. Each layer runs at twice the frequency and half the
amplitude of the last.

1. The gradient noise blends its eight lattice corners with the quintic
   `t * t * t * (t * (t * 6 - 15) + 10)` and sums the corner terms in the
   article's expanded order. Each corner takes the direction at its hash modulo
   12 among `[1, 1, 0]`, `[-1, 1, 0]`, `[1, -1, 0]`, `[-1, -1, 0]`,
   `[1, 0, 1]`, `[-1, 0, 1]`, `[1, 0, -1]`, `[-1, 0, -1]`, `[0, 1, 1]`,
   `[0, -1, 1]`, `[0, 1, -1]`, and `[0, -1, -1]`, each divided by `sqrt(2)`.
2. The hash runs on 32-bit unsigned integers with wrapping multiplication. The
   hash reads the seed and the lattice coordinates as 32-bit two's-complement
   integers. Octave `i` hashes with `seed + i`. The hash starts at
   `lowbias32(seed)` and folds in each coordinate in x, y, z order as
   `lowbias32(hash ^ coordinate)`. The
   [lowbias32](https://github.com/skeeto/hash-prospector) mixer XORs the value
   with itself shifted right by 16, multiplies by `0x7feb352d`, XORs with a
   shift by 15, multiplies by `0x846ca68b`, and XORs with a shift by 16.
3. Each octave after the first turns the point by the rotation with rows
   `[0, 0.8, 0.6]`, `[-0.8, 0.36, -0.48]`, and `[-0.6, -0.48, 0.64]` before
   doubling it. Gradient noise reads zero at every lattice point, and the turn
   keeps the octaves' lattices from lining up.
   [More noise](https://iquilezles.org/articles/morenoise/) explains the effect.
4. The raw sum `s` has the standard deviation
   `sigma = sigma1 * sqrt(sum of the squared amplitudes)`. `sigma1` reads
   0.1816, the deviation of one layer measured over 20 million points spread
   evenly through the lattice. `fbm` returns the algebraic
   [sigmoid](https://iquilezles.org/articles/sigmoids/) `x / sqrt(1 + x * x)` of
   `x = 0.8 * s / sigma`. The sigmoid spreads the values about evenly across
   `(-1, 1)`.

## Bounds

Every shape carries an axis-aligned box that holds every point where `d <= 0`.
Once the sampling settles, each 3D shape's box rounds out to the lattice. The
rounding takes the box moved by `o`: its min corner rounds down and its max
corner rounds up to the cell corners. Every 3D shape's distance then takes the
larger of its formula's value and the signed distance to its rounded box. A 2D
shape's box only feeds the boxes built from it.

The clamp leaves every sign alone and changes an exact distance nowhere. The
clamp also confines the points where `d <= r` to the box grown by `r`. The
growth rules below therefore hold for bounds and estimates alike. A box too
small would lose cells. Every rule below errs large.

1. A primitive's box wraps it tightly. Where its axis runs at an angle, the box
   comes from the [3D boxes](https://iquilezles.org/articles/bboxes3d/)
   formulas.
   - A `sphere`, an `ellipsoid`, and an `octahedron` span their center plus and
     minus their radius or radii.
   - A `box` and a `boxFrame` span their corners, and a `pyramid` spans its base
     square up to its apex.
   - A `cylinder` and a `cone` take the box around their two end disks. An end
     of radius `r` spans its center plus and minus `r * sqrt(1 - n * n)` per
     axis, with `n` the unit axis.
   - A `capsule` spans its ends plus and minus its radius, and a `roundCone`
     takes the box around its two end spheres.
   - A cut or whole `torus` spans its center plus and minus
     `ringRadius * sqrt(1 - n * n) + tubeRadius` per axis.
   - In the plane, a `circle` and an `ellipse` span their center plus and minus
     their radius or radii, and a `rect` and an `arch` span their corners. An
     `ngon`, a `star`, and a `polygon` take the box around their vertices, and a
     `polyline` takes the box around its points grown by half its width.
   - An `arc` takes the box around its end points and the points at `radius` in
     each axis direction its span crosses. Half its width then grows that box. A
     `sector`'s box also holds its center, and a `vesica` spans `a` and `b`
     grown by half its width.
2. `union`, `mirror`, `repeat`, and `repeatPolar` take the box around their
   operands or copies. Each step of a `smoothUnion` fold grows the box around
   its two operands by `radius / 4`.
3. `intersect` and `smoothIntersect` take the overlap of their bounded operands.
   `subtract` and `smoothSubtract` take the base's box.
4. A transform takes the box around its child's eight transformed corners.
5. `offset` grows the box by a positive distance and keeps it for a negative
   one. `shell` keeps it, `displace` grows it by `amplitude`, and `elongate`
   grows it by half its lengths on each side.
6. `twist` takes the box around the cylinder about its axis line that holds the
   child's box. `bend` takes the box around the annulus sector the child's box
   bends into.
7. `extrude` takes the profile's box between `from` and `to`. `revolve` takes
   the box around the circle the profile's largest u sweeps, across the
   profile's v span.
8. `halfSpace` has no box and no clamp. An `intersect` with a bounded operand
   gives the result a box again.

## The grid

Each part's grid spans the cells of the box around the part's `add` shapes moved
by `o` and the cell of each of its `set` points. The cap check runs before
anything allocates. Cells outside a part's grid stay empty. `carve`, `paint`,
and `coat` evaluate only the grid's cells and can therefore take an unbounded
shape.

## Steps

Each cell of a part's grid holds nothing or a material and remembers the last
step that changed it. A step reaches only its own part's grid.

1. `add` sets every cell its shape covers to the material.
2. `carve` empties every cell its shape covers.
3. `paint` sets every live cell its shape covers to the material.
4. `coat` reads a snapshot of the grid taken before it runs. A live cell takes
   the material when one of the `depth` cells beyond it toward a listed side is
   empty or outside the grid. `within` narrows the result to the cells its shape
   covers.
5. `set` sets the cell `floor((p + o) / g)` of each point `p`.

The steps outside every part belong to the root part. A pattern evaluates per
cell from the cell's [frame position](#pattern-frames) as its step runs.

A step in several parts' lists runs in each part. A part's `offset` moves the
part's frame within its parent's frame.

After a part's last step, `--fill-mode surface` empties each of the part's live
cells that does not show. A live cell shows when it has an empty face neighbor,
or when its material is opaque and a face neighbor's material lets light
through. A material lets light through when its pass at normal incidence is
above zero in any channel. Cells outside the part's grid count as empty. The
document holds the grids the fill mode leaves.

## Pattern frames

Evaluating a shape at a point returns its distance and a frame position. The
frame position carries the point back through every transform and modifier to
the frame of the primitive that decides the distance.

1. A primitive returns the point it was evaluated at.
2. A transform or modifier passes on its child's frame position. `twist` and
   `bend` evaluate the child at the unwarped point, and a pattern therefore
   follows the warp.
3. `union` and `smoothUnion` pass on the frame position of the operand with the
   smallest distance among all their operands, and the earliest operand wins a
   tie. `intersect`, `subtract`, and their smooth variants pass on their first
   operand's frame position.
4. `extrude` and `revolve` pass on their 3D point from before the plane mapping.

A `coat` or a `set` pattern reads the cell's center as a primitive returns it:
`[i + 0.5, j + 0.5, k + 0.5] * g - o`.

## Patterns

In the formulas below, `q` is the frame position, `n` counts the materials, and
`mod` takes the floored remainder. Every `fbm` and hash runs on the pattern's
seed, and a seed left out reads 0. Every `fbm` sums 4 octaves unless a `noise`
sets `octaves`. `hash(x, ...)` folds its whole-number arguments into the
[noise](#noise) hash in order. `unit(h)` computes `h / 2^32`.

1. `bands` computes `s = q[axis] + warp * fbm(q / (4 * period))`, and the cell
   takes material `mod(floor(s / period), n)`.
2. `grain` computes `s = r + warp * fbm(q / (4 * period))`, where `r` is the
   length of `q` with its `axis` component set to 0. The cell takes material
   `mod(hash(floor(s / period)), n)`.
3. `gradient` computes
   `t = (q[axis] + warp * fbm(q / ((to - from) / 4)) - from) / (to - from)`, and
   the cell takes material `floor(t * n)` held within `[0, n - 1]`.
4. `noise` computes `t = (fbm(q / scale) + 1) / 2`, and the cell takes material
   `floor(t * n)` held within `[0, n - 1]`.
5. `cells` works in lattice units of `size`. The lattice cube `c` holds one
   feature point at
   `c + [unit(hash(c, 0)), unit(hash(c, 1)), unit(hash(c, 2))]`.
   - The search for the nearest feature point scans the 27 cubes around the
     cell's cube in raster order with x outermost, and the first strict minimum
     wins. The nearest point's cube `c` gives the cell material
     `mod(hash(c, 3), n)`.
   - With `a` the offset from the cell to the nearest point, `border` takes the
     cell when another point at offset `b` among the 125 cubes around the
     nearest point's cube gives
     `size * dot((a + b) / 2, (b - a) / length(b - a))` of half a voxel or less.
     The [Voronoi edges](https://iquilezles.org/articles/voronoilines/) article
     derives this distance.
   - Each offset adds its cube's offset and its jitter before subtracting the
     cell's position within the cell's cube.
6. `speckle` reads the cell's indices `[i, j, k]`. The cell takes an accent when
   `unit(hash(i, j, k, 0))` reads below `density`. With `m` accents, the cell
   takes accent `mod(hash(i, j, k, 1), m)`.
7. `checker` gives the cell material
   `mod(floor(q[0] / size) + floor(q[1] / size) + floor(q[2] / size), n)`.

A pattern length left out takes its default count of cells times `g`.

## Materials

1. A `#RRGGBB` color decodes as sRGB and passes through the sRGB transfer into
   linear light. A `baseColor` alpha stays straight and skips the transfer. A
   `baseColor` without alpha takes an alpha of 1.
2. A named property a material leaves out takes its glTF default, except that
   `metallic` takes 0.
3. A custom property takes its kind from its value. A boolean writes `bool`, a
   string `string`, a number `float`, and an array of 2 to 4 numbers
   `vec-N-float`. `int` writes `int` or `vec-N-int`, and `json` writes `json`. A
   material leaving a custom property out takes the kind's empty value: 0, a
   zero vector, `false`, `""`, or `null`.
4. `shades` steps the base color's
   [Oklab](https://bottosson.github.io/posts/oklab/) lightness. Shade `i` of
   `count` steps the lightness by `(i - (count - 1) / 2) * spread`. A step of 0
   keeps the base color. A step down scales the color in linear light toward
   black by the amount that reaches the stepped lightness. A step up keeps the
   color's Oklab hue and chroma at the stepped lightness. A stepped lightness
   outside [0, 1] errors, and so does a step up that linear sRGB cannot hold at
   the color's chroma. The error gives the largest spread at which all `count`
   shades fit and rounds that spread down to three decimals. Every other
   property and the alpha carry over.
5. Two materials with identical properties merge into one palette material.

## The voxj document

1. Each root part writes a root node at its pivot, in meters, that scales by `g`
   on every axis. Coordinates under a root node count voxels, and the placed
   document therefore measures meters.
2. Under `--frame world`, each place of a part writes one object holding the
   place's live cells. Under `--frame local`, each part writes one object
   holding the part's live cells, and every place shares the object. Under
   `--flatten objects`, each root part writes one object holding every live cell
   at or below the root part instead. Where two parts cover one cell, the part
   placed later wins. An object takes its part's name. The object's `bounds`
   wrap its cells tightly. A grid with no live cell writes no object.
3. Each place of a part writes a node named after the part, and the nodes nest
   as the places do. Every node below a root node takes the position
   `(pivot - parentPivot) / g` and a scale of 1. Each pivot there sits where the
   offsets on its path move it. Every node takes its part's `rotation`, whose
   axis rotations apply about the parent's axes in list order. Under
   `--flatten nodes` or `--flatten objects`, only the root parts write nodes. A
   root part's node then places every object at or below the root part as the
   root part's own. Flattening a rotated part below a root part errors.
4. With `f = -(pivot + o) / g` and the pivot in the part's frame, a part's node
   places the part's object with an `origin` of the object's min cell index plus
   `f` when `f` is whole on every axis. Otherwise a child node named `voxels` at
   `f - floor(f)` places the object with an `origin` of its min cell index plus
   `floor(f)`. Each voxel then lands at its model position, and turning a part's
   node turns its object about the pivot.
5. Every object's layer references the one palette. vxl writes the document with
   the run's `--format` and encoding flags.
6. The palette binds the eight named properties in the order `baseColor`,
   `metallic`, `roughness`, `emissiveColor`, `emissiveStrength`,
   `occlusionStrength`, `ior`, and `transmission`, then each custom property the
   materials set in name order. Each property binds its own value pool of
   distinct values. The palette's materials follow the order a raster scan first
   meets them, object by object, with x outermost and z innermost.
7. Nodes and objects follow the order the places run in, from the first root
   part. A shared object comes at its part's first place.
8. The document carries no `editState` and no `ext`.

## The report

1. The report reads each place's grid as the part's last step leaves it. Only
   the voxel counts read the grids `--fill-mode surface` leaves. The grids come
   from `--frame world` under either frame. The model's line comes first. Each
   place of a part then takes a part line. The part's steps follow in list
   order, and its child parts come after them. A part's steps and child parts
   indent one level past the part's line. A lone root part takes no line, and
   its steps and child parts follow the model's line. A part's steps print under
   its first place only, and each later place's line reads `repeats` and the
   first place's path.
2. An `add` or a `set` writes every cell it fills. A `carve` writes the live
   cells it empties, and a `paint` or a `coat` writes the live cells it
   recolors. A step keeps the cells whose last change came from the step.
   `exposed` counts the kept live cells that show, as the surface fill reads
   them. A step's size and bounds cover the cells the step wrote.
3. A step other than a `carve` gives the count of face-connected pieces among
   its written cells that hold a material, when there is more than one. A
   step line ends with `lost to` and the later steps whose writes changed the
   step's cells, when any did. A `paint` or a `coat` line lists after `over` the
   earlier steps whose cells it wrote over, when it wrote over any. Both lists
   run in list order.
4. A part line's count, size, and bounds cover the part's live cells at that
   place, and its pivot gives the place's pivot in meters. A part with live
   cells reads `detached` when its parent has live cells and no piece holding
   the cells of the part or a part below it holds a parent cell. The model's
   line and the pieces cover the cells the placed parts cover together.
5. Pieces are the face-connected groups of the cells the placed parts cover. The
   report numbers the pieces from the largest. Among pieces of one size, the
   piece whose first cell comes first in a raster scan with x outermost takes
   the lower number. With more than one piece, the report ends with a line per
   piece that lists in list order each step with a written cell in the piece
   that holds a material. A step name that repeats across lists takes its path
   of part names. The path leaves out a lone root part.
6. A size counts cells along x, y, and z. Bounds run in meters from the min
   cell's min corner to the max cell's max corner. A line over no cells leaves
   out its size and bounds. The report rounds meters to six decimals, drops
   trailing zeros, and prints `-0` as `0`.
7. A count of one takes a singular label, such as `1 cell`. The report pads the
   fields of consecutive lines of one kind and depth into columns and aligns
   the counts right. A column no line of a group fills takes no space.

## Checks

Each command stops at its first failed check. A voxelize check reports the step
it belongs to by its path of part names.

1. `vxl sdf-doc build` checks its flags and profile. `--runtime` takes `node`,
   `bun`, or `deno`, and `--profile` reads a name the cascade holds. `--library`
   and `libraries` read names the cascade defines, and each library they list
   follows the sdfj format's rules. A runtime that fails to start stops the
   build with the system's message.
2. While the model builds, the builder checks the default export and what the
   document cannot hold. The default export is an array of steps and parts.
   Each argument takes the type the modeling API declares, and an options
   object holds only the options its call lists. Numbers are finite, `json`
   holds only JSON values, and `lib` and `mat` read only the names the
   libraries hold. `shades` takes a whole count above zero, and a boolean takes
   at least one shape. Each named export holds a material, a pattern, a shape,
   a step, a part, or a function. Two different entries of one kind take
   different names. A library part's node holds at most one object. That object
   takes the node's name, lists at least one step, and belongs to no other
   node. An error the model throws stops the build with its message.
3. `vxl sdf-doc voxelize` checks its flags and profile against the values
   `vxl mesh-doc voxelize` lists. `--voxel-size` reads above zero,
   `--resolution` takes a whole number above zero, and the two flags exclude
   each other. `--flatten nodes` and `--flatten objects` need `--frame world`.
4. vxl reads the document by the
   [sdfj format](../../../projects/sdf-formats/sdfj/docs/sdf-json-file-format.md#rules)
   and checks every entry's arguments.
   - Radii, widths, thicknesses, sizes, scales, periods, and `spread` are above
     zero, except that a `cone` end may take 0. A `round` and a `chamfer` are
     zero or more.
   - Counts, `octaves`, and `depth` are whole numbers above zero. An `ngon`'s
     `sides` is at least 3 and a `star`'s `points` at least 2. A seed is a
     whole number from `-2^31` to `2^31 - 1`.
   - A pattern's `materials` and a `speckle`'s `accents` hold at least one
     material. A `union`, an `intersect`, and their smooth variants hold at
     least one shape. A `polygon` lists at least 3 points, and a `polyline` and
     a `lathe` list at least 2.
   - `amplitude`, `warp`, and the lengths of an `elongate` are zero or more,
     `density` falls in `[0, 1]`, and a range's end passes its start.
   - Colors parse as `#RRGGBB`, or `#RRGGBBAA` for `baseColor`. Named properties
     fall in their ranges, and a shade stays inside `[0, 1]` in linear light.
   - A custom property takes a non-empty name and a value with a kind. An `int`
     holds whole numbers within `2^53 - 1`, and a vector holds 2 to 4 numbers.
   - The corners of a `box`, a `boxFrame`, and a `rect` differ on each axis and
     may come in either order. A `rect` takes `round` or `chamfer` but not both,
     and either stays within half its shorter side. A `box`'s `round` stays
     within half its shortest extent, and a `cylinder`'s within its radius and
     half its length.
   - The ends of a `cylinder`, a `cone`, a `capsule`, a `roundCone`, and a
     `vesica` differ, as do a `polyline`'s neighboring points. A `roundCone`'s
     radii differ by less than the distance between its ends.
   - A `star`'s inner radius stays below its outer radius, and an `arch` stands
     at least half its width tall. An `arc`, a `sector`, and a cut `torus` span
     at most 360 degrees. A `torus` takes both `from` and `to` or neither.
   - `orient` takes two directions of nonzero length that do not point opposite
     ways. An `elongate`'s `center` lies inside its shape's box along each axis
     it stretches. A `bend`'s `along` and `toward` lie on different axes, and
     its shape has a box.
   - A `polygon` outline never crosses itself.
5. Over the document, every step has a non-empty name no other step in its list
   shares, and every part has a non-empty name no other part in its list shares.
   The root parts count as one list. Every material that sets a custom property
   gives it one kind.
6. Over the sampling and the grid, every `add` shape has a box, and a
   `--resolution` reference side reads above zero. Each part's grid stays within
   2^27 cells, and so does each root part's object under `--flatten objects`. No
   `bend` reaches past half a turn of its arc: its shape's box spans at most
   `pi * radius` along `along`.
7. While the steps run, no distance reads NaN, and no `set` lists the same point
   twice.
8. At the end, the model holds a live cell.

A step that writes no cell and a part with no live cell show in the report and
stop nothing.
