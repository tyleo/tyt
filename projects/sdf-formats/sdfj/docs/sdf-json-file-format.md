# SDF Json File Format

A JSON format for solid models built from signed distance functions: shapes,
materials, patterns, the steps that fill each part with them, and the parts'
hierarchy. Examples below are JSONC for readability; real files are plain JSON.

## Design Objectives

1. **Human-understandable:** the format is plain JSON, and each entry holds one
   shape, material, pattern, or step with its arguments by name.
2. **Exactly defined:** the format sets the value every entry takes at any
   point, so a model stands apart from any one sampling of it.
3. **Shared:** an entry appears once and other entries reference it by index,
   so a shape feeding several steps or a part standing in several spots costs
   one entry. Names let one document read another's entries.

Most formulas come from Inigo Quilez's MIT-licensed
[articles](https://iquilezles.org/articles/). Each section links the articles it
follows.

## File Extensions

An SDF Json file is a single JSON document. It is stored in one of two
interchangeable forms with identical content, differing only in whether the
document is packaged in a zip archive.

| Form         | Extension | Contents                                                  | MIME type                   |
| ------------ | --------- | --------------------------------------------------------- | --------------------------- |
| Uncompressed | `.sdfj`   | The JSON document as UTF-8 text, read and edited directly | `application/vnd.sdfj+json` |
| Compressed   | `.sdfjz`  | A zip archive containing the document                     | `application/vnd.sdfj+zip`  |

1. `.sdfj` is the canonical working form. It is plain UTF-8 JSON whose
   top-level value is the object in [Structure](#structure), openable in any
   JSON-aware editor. Tools that key off the `.json` suffix may also accept
   `.sdfj.json`.
2. `.sdfjz` is the canonical shipping form. It is a standard zip archive holding
   exactly one `.sdfj` member, conventionally named `main.sdfj`, compressed with
   the deflate method. The archive may carry additional resource members;
   consumers ignore members they do not recognize. Reading a `.sdfjz` means
   opening the archive and parsing its `.sdfj` member, which is byte-identical
   to a standalone `.sdfj`. Tools that key off the `.zip` suffix may also accept
   `.sdfjz.zip`.
3. Packaging is a transport concern and never changes the document. Recognize
   the form by leading bytes, not extension. An uncompressed file's first
   non-whitespace byte is `{`. A zip archive begins with `PK`, the bytes
   `0x50 0x4B`. Consumers accept either form wherever an SDF Json file is
   expected.

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

  "fonts": [
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

Each table holds one kind of entry, and an entry references another by its
index in that entry's table.

1. `shapes3d` and `shapes2d` hold the [shapes](#shapes), and `fonts` the
   [fonts](#fonts) a `text` reads
2. `materials` and `shades` hold the [materials](#materials)
3. `patterns` holds the [patterns](#patterns), and `steps` the [steps](#steps)
4. `objects` and `nodes` hold the [parts](#parts), and `rootNodes` lists the
   nodes at the top of the hierarchy
5. `names` gives entries the [names](#names) other documents read them by

A document holds `version` and can leave out any other key. A missing table or
names map reads as empty.

In every entry but a `shades` entry, an object, and a node, `kind` picks what
the entry computes. An optional key takes the default its comment gives. A
point or a direction writes an array: `[x, y, z]` in 3D and `[u, v]` in the
plane. Every number reads as a 64-bit float.

## Coordinate System

The axes are glTF's: Y is up, the frame is right-handed, and +Z points toward
the viewer. Lengths count meters and angles count degrees. A positive rotation
turns counterclockwise seen from the positive end of its axis.

Each part has a frame. The part's shapes, its `set` points, and its `pivot` take
coordinates in that frame. A root part sits in the model frame, and a child part
sits in its parent's frame. With `R` the part's rotation, a point `x` in the
part's frame lands at `offset + pivot + R(x - pivot)` in the frame above. The
part therefore turns about its pivot and then moves by its offset. `R` applies
the axis rotations in list order, each about an axis of the frame above.

A 2D shape lies in a plane with axes u and v, and its angles run from +u toward
+v. Each [3D shape built from a profile](#from-2d-to-3d) sets where its plane
sits.

## Parts

An object holds a part's steps, and a node places a part.

```jsonc
{
  "objects": [
    {
      "name": "tree",

      // indices into `steps`, in the order the steps run
      "steps": [0, 1],
    },
  ],

  "nodes": [
    {
      "name": "tree.1",

      // optional: the point the part turns about, in the part's frame;
      // defaults to [0, 0, 0]
      "pivot": [0, 0, 0],

      // optional: moves the part, pivot included, within the frame above;
      // defaults to [0, 0, 0]
      "offset": [-0.75, 0, 0],

      // optional: turns the part about its pivot, one axis rotation at a time
      // in list order; defaults to []
      "rotation": [{ "axis": "y", "degrees": 30 }],

      // optional: marks a part that moves on its own, such as a lid or a
      // door; defaults to false
      "joint": true,

      // indices into `objects`: the part's steps
      "childObjects": [0],

      // indices into `nodes`: the child parts
      "childNodes": [],
    },
  ],
}
```

1. Each object fills the part's frame with materials by its [steps](#steps).
   One object's steps read and write only that object, and objects overlap
   freely. How overlapping objects combine is up to the reader
2. A node reached through several parents places its part once per path
3. `rootNodes` lists the parts that sit in the model frame
4. A reader that merges parts into fewer objects keeps each joint apart from
   its parent, so the joint can still turn

## Steps

An object's steps fill its frame in list order. A point starts empty and holds
nothing or one material. A point that holds a material reads live, and each
live point remembers its owner: the last `add` that filled it.

```jsonc
[
  // add: fills the points inside `shape`
  {
    "kind": "add",

    // identifies the step within its object
    "name": "seat",

    // an index into `shapes3d`
    "shape": 3,

    // an index into `materials`. A step that takes a material holds either
    // `material` or `pattern`, an index into `patterns`
    "material": 0,
  },

  // carve: empties the points inside `shape`
  { "kind": "carve", "name": "hole", "shape": 4 },

  // paint: recolors the live points inside `shape`
  {
    "kind": "paint",
    "name": "trim",
    "shape": 5,
    "pattern": 1,

    // optional: indices into `steps`, the `add` steps whose points the step
    // keeps to; defaults to every point
    "onto": [0],
  },

  // coat: recolors the live points near the surface
  {
    "kind": "coat",
    "name": "moss",
    "material": 2,

    // optional: the sides to look toward; defaults to all six
    "sides": ["+y"],

    // how far to look
    "depth": 0.025,

    // optional: an index into `shapes3d` that bounds the coat; defaults to
    // every point
    "within": 6,

    "onto": [0],
  },

  // set: fills a cube around each point
  {
    "kind": "set",
    "name": "eyes",
    "points": [
      [-0.05, 0.4, 0.1],
      [0.05, 0.4, 0.1],
    ],

    // the cubes' width
    "size": 0.025,

    "material": 3,
  },
]
```

A point lies inside a shape where the shape's distance reads zero or less.

1. `add` gives every point inside its shape the material, and the `add` becomes
   the point's owner
2. `carve` empties every point inside its shape and clears the point's owner
3. `paint` gives every live point inside its shape the material
4. `coat` gives a live point `x` the material when, for some listed side with
   direction `n`, the point `x + depth * n` lies outside the object's earlier
   shapes. The last `add` or `carve` before the `coat` whose shape holds that
   point decides: inside after an `add` and outside after a `carve`. A point no
   such shape holds lies outside. `within` narrows the result to the points
   inside its shape
5. `set` gives the material to every point `x` with
   `p - size / 2 <= x < p + size / 2` on every axis, for each listed point `p`.
   Each point keeps its owner

`onto` narrows a `paint` or a `coat` to the points the listed `add` steps own. A
[pattern](#patterns) picks its material per point.

## Shapes

Each shape is a signed distance function `d(p)` in meters that reads negative
inside, positive outside, and zero on the surface. Every primitive gives an
exact signed distance unless its line says otherwise. Only the sign decides
which points a step reaches. `offset`, `shell`, and the smooth booleans read the
distance's size. The formulas hold in exact arithmetic, and a reader's rounding
can tip a point within rounding of a surface either way.

The [3D](https://iquilezles.org/articles/distfunctions/) and
[2D](https://iquilezles.org/articles/distfunctions2d/) catalogs supply the
formulas the lines below cite.

### 3D primitives

```jsonc
[
  // box: the box between two corners
  {
    "kind": "box",

    // opposite corners, in either order on each axis
    "min": [-0.25, 0, -0.25],
    "max": [0.25, 0.5, 0.25],

    // optional: rounds the edges and corners by this radius inside the same
    // corners; defaults to 0
    "round": 0.02,
  },

  // boxFrame: the twelve edges of the box between two corners, as square bars
  // that stay inside the corners
  {
    "kind": "boxFrame",
    "min": [-0.25, 0, -0.25],
    "max": [0.25, 0.5, 0.25],

    // the bars' width
    "thickness": 0.025,
  },

  { "kind": "sphere", "center": [0, 0.5, 0], "radius": 0.5 },

  // ellipsoid: `radii` along x, y, and z
  { "kind": "ellipsoid", "center": [0, 0.5, 0], "radii": [0.5, 0.25, 0.5] },

  // cylinder: from `a` to `b` at any angle
  {
    "kind": "cylinder",
    "a": [0, 0, 0],
    "b": [0, 1, 0],
    "radius": 0.1,

    // optional: rounds the rims by this radius; defaults to 0
    "round": 0.01,
  },

  // cone: `radiusA` at `a` and `radiusB` at `b`; a radius of 0 makes a point
  {
    "kind": "cone",
    "a": [0, 0, 0],
    "b": [0, 1, 0],
    "radiusA": 0.25,
    "radiusB": 0,
  },

  // roundCone: a sphere of each end's radius at each end, joined by their
  // outer tangents
  {
    "kind": "roundCone",
    "a": [0, 0, 0],
    "b": [0, 1, 0],
    "radiusA": 0.1,
    "radiusB": 0.05,
  },

  // capsule: the points within `radius` of the segment from `a` to `b`
  { "kind": "capsule", "a": [0, 0, 0], "b": [0, 1, 0], "radius": 0.05 },

  // torus: a tube around a ring
  {
    "kind": "torus",
    "center": [0, 0, 0],

    // from the center to the middle of the tube
    "ringRadius": 0.5,

    "tubeRadius": 0.1,

    // optional: the axis the ring circles; defaults to "y"
    "axis": "y",

    // optional, as a pair: cut the ring to the arc from `from` to `to`
    // degrees, with round ends. The angles run in the plane `extrude` maps
    // across `axis`. Default to the whole ring
    "from": 0,
    "to": 180,
  },

  // octahedron: reaches `radius` along each axis from `center`
  { "kind": "octahedron", "center": [0, 0.5, 0], "radius": 0.5 },

  // pyramid: a square base `width` across centered on `baseCenter`, with its
  // apex `height` above along +y
  { "kind": "pyramid", "baseCenter": [0, 0, 0], "width": 1, "height": 0.75 },

  // halfSpace: every point past `at` toward `side`; this one holds every
  // point above y = 0.25
  { "kind": "halfSpace", "side": "+y", "at": 0.25 },
]
```

1. `sphere` gives `length(p - center) - radius`
2. With `h` its half extents, `box` folds the point about its center into
   `q = abs(p - center) - h` and gives `length(max(q, 0)) + min(max(q), 0)`.
   `round` shrinks `h` by the radius and subtracts the radius
3. `boxFrame` takes the least box distance among its three bar directions after
   folding the point into one octant. The frame reads short inside where its
   bars meet
4. `cylinder`, `cone`, `capsule`, and `roundCone` measure in the half-plane
   through their axis. The point's radial distance `k` takes the length of its
   offset from the axis line and never comes from a difference of squares. `v`
   gives the point's position along the axis
   - A `cylinder` gives the box distance of `[k, v]` with half extents
     `[radius, length / 2]` and `v` measured from the midpoint. `round` shrinks
     and subtracts the same way
   - A `cone` measures to the trapezoid between its end radii by the capped cone
     formula
   - A `capsule` measures to its segment and subtracts the radius
   - A `roundCone` measures to its two end circles and their outer tangents by
     the round cone formula
5. `torus` gives `length([k - ringRadius, v]) - tubeRadius` about its axis. A
   cut torus turns the plane so the arc's middle lies on +v and folds u to
   `abs(u)`. A point past the arc's end then measures to the ring's end point
6. `octahedron` uses the exact octahedron formula and not the bound
7. `pyramid` evaluates the unit-base pyramid at `(p - baseCenter) / width` with
   height `height / width` and multiplies the result by `width`. The pyramid
   formula measures to the slanted faces. At or below the base plane the
   distance runs to the base square instead, and inside it takes the nearer of
   the faces and the base
8. `ellipsoid` gives a bound from the
   [ellipsoid article](https://iquilezles.org/articles/ellipsoids/). With
   `q = p - center`, `k0 = length(q / radii)`, and
   `k1 = length(q / (radii * radii))`, it reads `k0 * (k0 - 1) / k1` where
   `k0 >= 1` and `(k0 - 1) * min(radii)` elsewhere. The first stays tight
   outside, and the second never reads deeper than the true depth or divides by
   zero at the center
9. `halfSpace` gives the signed distance to its plane

### 2D primitives

```jsonc
[
  { "kind": "circle", "center": [0, 0], "radius": 0.5 },

  // rect: the rectangle between two corners, in either order on each axis
  {
    "kind": "rect",
    "min": [-0.5, -0.25],
    "max": [0.5, 0.25],

    // optional: cuts each corner at 45 degrees with legs this long; defaults
    // to 0
    "chamfer": 0.05,

    // optional: rounds each corner by this radius; defaults to 0
    "round": 0,
  },

  // ellipse: `radii` along u and v
  { "kind": "ellipse", "center": [0, 0], "radii": [0.5, 0.25] },

  // ngon: `sides` sides, each `radius` from `center`, with one side facing -v
  { "kind": "ngon", "center": [0, 0], "sides": 6, "radius": 0.5 },

  // star: `points` tips at `outerRadius` with the first at +v, and inner
  // vertices at `innerRadius` halfway between
  {
    "kind": "star",
    "center": [0, 0],
    "points": 5,
    "outerRadius": 0.5,
    "innerRadius": 0.2,
  },

  // polygon: the outline through `points`, closed back to the first
  {
    "kind": "polygon",
    "points": [
      [0, 0],
      [1, 0],
      [0, 1],
    ],
  },

  // polyline: a stroke `width` across through `points`
  {
    "kind": "polyline",
    "points": [
      [0, 0],
      [1, 0],
    ],
    "width": 0.05,
  },

  // spline: a smooth curve through every point
  {
    "kind": "spline",
    "points": [
      [0, 0],
      [0.5, 0.25],
      [1, 0],
    ],

    // optional: strokes the curve open at this width; without it the curve
    // closes into an outline
    "width": 0.05,
  },

  // arc: a stroke `width` across along the circle of `radius` from
  // `fromDegrees` to `toDegrees`
  {
    "kind": "arc",
    "center": [0, 0],
    "radius": 0.5,
    "fromDegrees": 0,
    "toDegrees": 180,
    "width": 0.05,

    // optional: "flat" or "round" ends; defaults to "round"
    "caps": "flat",
  },

  // sector: the pie slice of the disk of `radius` from `fromDegrees` to
  // `toDegrees`
  {
    "kind": "sector",
    "center": [0, 0],
    "radius": 0.5,
    "fromDegrees": 0,
    "toDegrees": 90,
  },

  // vesica: a pointed lens `width` across from `a` to `b`
  { "kind": "vesica", "a": [0, 0], "b": [0, 1], "width": 0.4 },

  // arch: the rectangle between two corners topped by a half circle as wide
  // as the rectangle, inside the same corners
  { "kind": "arch", "min": [-0.5, 0], "max": [0.5, 1.5] },

  // text: a line of text set in a font with its kerning, along +u with its
  // baseline on v = 0
  {
    "kind": "text",

    // an index into `fonts`
    "font": 0,

    "string": "OPEN",

    // the height of a capital letter
    "size": 0.1,

    // optional: defaults to "start". "start", "center", or "end" picks the
    // point of the line that sits on u = 0
    "align": "center",
  },
]
```

1. `circle` gives `length(p - center) - radius`
2. `rect` gives the box distance in the plane. `round` shrinks and subtracts the
   same way, and `chamfer` uses the chamfer box formula
3. `ellipse` runs the trig-free Newton iteration from the
   [ellipse article](https://iquilezles.org/articles/ellipsedist/) exactly 10
   times from its starting guess. The iteration refines the nearest point's
   cosine and sine, and the distance runs to the point it lands on. With radii
   `a` and `b`, the sign comes from the division-free test
   `b * b * x * x + a * a * y * y <= a * a * b * b`. Equal radii take the
   circle's formula
4. `ngon` and `star` fold the point into one sector. Each builds its directions
   once: the side midpoints of an `ngon` and the tips of a `star`. A point turns
   by the rotation that takes its direction of largest dot product onto +v, and
   the earliest direction wins a tie. The point then folds u to `abs(u)`
   - An `ngon` measures to its one side at `v = radius` and reads inside where
     `v <= radius`
   - A `star` measures to the edge from its tip at `[0, outerRadius]` to the
     inner vertex at `innerRadius` halfway to the next tip. It reads inside on
     the center's side of that edge
5. `polygon` measures to its nearest edge through the least squared edge
   distance and one square root. Its sign follows the even-odd rule along +u.
   With `e = v[j] - v[i]` for the edge from `v[i]` back to the previous vertex
   `v[j]` and `w = p - v[i]`, the edge flips the sign when the tests
   `p.v >= v[i].v`, `p.v < v[j].v`, and `e.u * w.v > e.v * w.u` all hold or all
   fail
6. `polyline` measures to its nearest segment the same way and subtracts half
   the width
7. `spline` replaces its points with points along the centripetal Catmull-Rom
   curve through them. It then measures as a `polygon` without `width` and as a
   `polyline` with it
   - Each span between neighboring points splits into 16 segments at even steps
     of its knot parameter. The knots lie apart by the square root of the
     distance between neighboring points
   - A closed spline's spans wrap around to the first point. An open one adds
     `2 * p[0] - p[1]` before its first point and the same reflection after its
     last
8. `arc` turns the plane so the arc's middle lies on +v and folds u to `abs(u)`.
   With `s` and `c` the sine and cosine of half its span, a point where
   `c * u > s * v` measures to the arc's end point, and any other point to the
   circle as `abs(length(p) - radius)`. Both then subtract half the width. Flat
   caps instead cut the band square at the ends by the ring formula
9. `sector` folds the same way and takes the larger of the circle's distance and
   the signed distance to the end's radius segment by the pie formula
10. `vesica` uses the oriented vesica formula with half its width
11. `arch` measures to its outline of base, sides, and half circle by the tunnel
    formula
12. `text` sets each character's outline from its font, scaled by `size` over
    the font's cap height. A font without a cap height takes the top of its `H`
    - Each character starts where the previous character's advance ends.
      `align` shifts the line back by none, half, or all of the advances' sum
    - Each cubic curve of the font splits into quadratic curves that stray from
      it by at most a thousandth of an em
    - The text measures to its nearest segment. A line measures as a `polygon`
      edge does. A curve `b(t)` with `t` in `[0, 1]` measures to the nearest of
      its ends and the roots of the cubic `dot(b(t) - p, b'(t)) = 0`
    - The sign follows the nonzero rule on the ray from `p` toward +u. A curve
      splits into pieces where its v turns
    - Each line or piece that crosses the ray counts +1 when it runs up in v and
      -1 when it runs down. It crosses when `p.v` lies in the half-open range
      from its start's v to its end's v and it meets that v right of `p`
    - Inside an overlap of contours, the distance runs to the nearest segment
      even when that segment lies inside the fill

### From 2D to 3D

```jsonc
[
  // extrude: pushes the profile from `from` to `to` along `axis`
  {
    "kind": "extrude",

    // an index into `shapes2d`
    "profile": 0,

    // optional: defaults to "z". The profile's u and v map to the other two
    // axes in x, y, z order: (x, y) for "z", (x, z) for "y", and (y, z) for
    // "x"
    "axis": "z",

    "from": 0,
    "to": 0.1,
  },

  // revolve: spins the profile around the line along `axis` through `center`.
  // The profile's u measures the distance from the line and v the position
  // along it from `center`, and only the half at u >= 0 counts
  {
    "kind": "revolve",
    "profile": 0,

    // optional: defaults to "y"
    "axis": "y",

    // optional: defaults to [0, 0, 0]
    "center": [0, 0, 0],
  },

  // lathe: revolves the outline its `[radius, height]` points trace, closed
  // back along the axis; takes `axis` and `center` as `revolve` does
  {
    "kind": "lathe",
    "points": [
      [0.05, 0],
      [0.03, 0.4],
    ],
  },

  // sweep: runs the profile along a smooth curve through `points`. The
  // profile's origin rides the curve. Facing along the curve, v points as
  // near `up` as the curve allows and u points right. The ends cut flat
  {
    "kind": "sweep",
    "profile": 0,
    "points": [
      [0, 0, 0],
      [0, 0.5, 0.25],
      [0, 1, 0],
    ],

    // optional: the profile's scale at each point; defaults to 1 at each
    "scales": [1, 0.75, 0.5],

    // optional: defaults to [0, 1, 0]
    "up": [0, 0, 1],
  },
]
```

An exact profile gives an exact extrusion. A revolution stays exact when its
profile keeps clear of the axis or is symmetric about it, and reads short near
the axis otherwise.

1. `extrude` combines the profile's distance `d2` with the distance `h` to the
   slab between `from` and `to` as
   `min(max(d2, h), 0) + length([max(d2, 0), max(h, 0)])`
2. `revolve` evaluates the profile at `[k, v]`, with `k` the point's distance
   from the axis and `v` its position along the axis from `center`
3. `lathe` revolves the `polygon` of its points followed by their mirror images
   across the axis in reverse order. The outline closes across the axis at its
   first and last heights, and no edge runs along the axis
4. `sweep` samples the curve of an open `spline` through its points with each
   point's scale as a fourth coordinate. The knots follow the 3D distances
   alone. A sample's tangent `t` points from the sample before it to the sample
   after it, or along the end segment at either end
   - The nearest point `c` on the sampled segments sets the frame. Along that
     segment, `t` and the scale `s` blend linearly. The frame takes
     `u = normalize(cross(t, up))` and `v = cross(u, t)`
   - The profile reads `s * d2(q / s)` at `q = [dot(p - c, u), dot(p - c, v)]`,
     or `length(q)` where `s` reads 0
   - Where `c` lands on an end of the curve, the profile combines with
     `abs(dot(p - c, t))` by the `extrude` formula. Elsewhere the profile's
     distance stands alone and overstates the depth inside near the ends

### Booleans

The booleans take the same keys in `shapes3d` and `shapes2d`, and their indices
point into the entry's own table.

```jsonc
[
  // union, intersect: the points inside any or all of `shapes`
  { "kind": "union", "shapes": [0, 1, 2] },
  { "kind": "intersect", "shapes": [0, 1] },

  // subtract: `base` less the points inside any of `cutters`
  { "kind": "subtract", "base": 0, "cutters": [1, 2] },

  // the smooth variants blend where their operands meet over `radius`
  { "kind": "smoothUnion", "radius": 0.05, "shapes": [0, 1] },
  { "kind": "smoothIntersect", "radius": 0.05, "shapes": [0, 1] },
  { "kind": "smoothSubtract", "radius": 0.05, "base": 0, "cutters": [1] },
]
```

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

More than two operands fold left, and a `subtract` folds its cutters onto its
base in list order. The smooth folds depend on the order.

A `union` is exact outside and reads short inside. Inside, a `union` measures to
the nearest operand's face even when another operand buries that face.
[Interior distances](https://iquilezles.org/articles/interiordistance/) covers
the effect. An `intersect` and a `subtract` are exact inside and read short
outside. On exact operands the smooth variants never read long and read exact
where `abs(a - b) >= r`.

### Transforms

```jsonc
[
  // translate: moves `shape`, an index into the entry's table, by `offset`
  { "kind": "translate", "shape": 0, "offset": [0.2, 0, 0.2] },

  // rotate: turns `shape` by `degrees` about the line along `axis` through
  // `pivot`
  {
    "kind": "rotate",
    "shape": 0,
    "axis": "y",
    "degrees": 90,

    // optional: defaults to [0, 0, 0]
    "pivot": [0, 0, 0],
  },

  // orient: turns `shape` about `pivot` until the direction `from` points
  // along `to`
  { "kind": "orient", "shape": 0, "from": [0, 1, 0], "to": [1, 1, 0] },

  // scale: multiplies `shape`'s offset from `pivot` by `factor` on each
  // axis; a negative factor reflects its axis
  { "kind": "scale", "shape": 0, "factor": [-1, 1, 1] },

  // mirror: `shape` and its reflections across `center` along each axis in
  // `axes`, one of "x", "y", "z", "xy", "xz", "yz", and "xyz"; "xz" gives
  // four copies
  {
    "kind": "mirror",
    "shape": 0,
    "axes": "xz",

    // optional: defaults to [0, 0, 0]
    "center": [0, 0, 0],
  },

  // repeat: `count` copies along each axis; the copy at [i, j, k] moves by
  // [i * step[0], j * step[1], k * step[2]]
  { "kind": "repeat", "shape": 0, "step": [0, 0, 0.3], "count": [1, 1, 5] },

  // repeatPolar: `count` copies turned evenly about the line along `axis`
  // through `center`; copy i turns by i * 360 / count degrees
  { "kind": "repeatPolar", "shape": 0, "axis": "y", "count": 6 },
]
```

`orient` and `scale` take an optional `pivot`, and `repeatPolar` an optional
`center`, that default to `[0, 0, 0]`. In `shapes2d` the transforms take 2D
points and no `axis`: `rotate` turns about `pivot` in the plane, `mirror` takes
`axes` of "u", "v", or "uv", and `orient` does not exist.

A transform evaluates its child at the point mapped back through its inverse.
Translation, rotation, and `orient` keep the distance exact. A scale `s` returns
`min(|s|) * d(p / s)`. The result reads exact when every factor shares one
magnitude and short otherwise.

`orient` normalizes its two directions once and builds its rotation from them
with the trig-free alignment from
[avoiding trigonometry](https://iquilezles.org/articles/noacos/).

`mirror`, `repeat`, and `repeatPolar` return the union of their copies: the
minimum of `d` over the point mapped back into the original by each reflection,
translation, or rotation. `repeatPolar` turns the point by each copy's rotation
rather than folding the point by its angle.

### Modifiers

```jsonc
[
  // offset: grows `shape` by `distance`, or shrinks it by a negative one
  { "kind": "offset", "shape": 0, "distance": 0.01 },

  // shell: keeps the outer `thickness` of `shape`
  { "kind": "shell", "shape": 0, "thickness": 0.025 },

  // elongate: cuts `shape` through `center` across each axis, moves the
  // halves apart by `lengths`, and fills the gap with the cut's cross-section
  {
    "kind": "elongate",
    "shape": 0,
    "lengths": [0.5, 0, 0],

    // optional: defaults to [0, 0, 0]
    "center": [0, 0, 0],
  },

  // twist: turns each slice across `axis` by `degreesPerMeter` times the
  // slice's offset from `center` along the axis; `center` defaults to
  // [0, 0, 0]
  { "kind": "twist", "shape": 0, "axis": "y", "degreesPerMeter": 90 },

  // bend: curls `shape`'s `along` axis into an arc of `radius` curving toward
  // `toward`. The slice through `pivot` stays put, and the arc keeps every
  // length along the axis. `pivot` defaults to [0, 0, 0]
  {
    "kind": "bend",
    "shape": 0,
    "along": "x",
    "toward": "+y",
    "radius": 1,
  },

  // displace: roughens the surface by fractal noise up to `amplitude` deep
  // with features about `scale` across
  {
    "kind": "displace",
    "shape": 0,
    "amplitude": 0.01,
    "scale": 0.1,

    // optional: the layers the noise sums; defaults to 4
    "octaves": 4,

    "seed": 7,
  },
]
```

`shapes2d` takes `offset` and `shell` with the same keys.

| Modifier    | Distance                         |
| ----------- | -------------------------------- |
| `offset(r)` | `d - r`                          |
| `shell(t)`  | `max(d, -d - t)`                 |
| `displace`  | `d + amplitude * fbm(p / scale)` |

`shell(t)` keeps the points with `-t <= d <= 0`. The `fbm` of a `displace` sums
its `octaves` and runs on its seed.

`elongate` evaluates its child at `p - clamp(p - center, -h, h)` with `h` half
its lengths. An `elongate` along one axis stays exact, and one along more axes
reads short inside.

`twist` turns the point about the axis line through `center` by
`-degreesPerMeter` times its offset along the axis from `center` and then
evaluates the child.

`bend` maps each point back to the unbent shape:

1. The arc's center `C` sits `radius` from `pivot` toward `toward`
2. The middle of the child's [box](#boxes) along `along` bends to the direction
   `m` from `C`. The angle from the direction of `pivot` to `m` is
   `phiM = (middle - pivot) / radius` and runs positive toward +`along`
3. With `u` the point's offset from `C` in the bend plane, the point's angle is
   `phiM` plus the signed angle from `m` to `u`. The signed angle takes the
   `atan2` of the cross and dot products of `m` and `u`
4. The point's angle times `radius` gives its unbent offset from `pivot` along
   `along`, and `radius - length(u)` gives its unbent offset from `pivot` toward
   `toward`. The third coordinate stays

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
   `[0, -1, 1]`, `[0, 1, -1]`, and `[0, -1, -1]`, each divided by `sqrt(2)`
2. The hash runs on 32-bit unsigned integers with wrapping multiplication. The
   hash reads the seed and the lattice coordinates as 32-bit two's-complement
   integers. Octave `i` hashes with `seed + i`. The hash starts at
   `lowbias32(seed)` and folds in each coordinate in x, y, z order as
   `lowbias32(hash ^ coordinate)`. The
   [lowbias32](https://github.com/skeeto/hash-prospector) mixer XORs the value
   with itself shifted right by 16, multiplies by `0x7feb352d`, XORs with a
   shift by 15, multiplies by `0x846ca68b`, and XORs with a shift by 16
3. Each octave after the first turns the point by the rotation with rows
   `[0, 0.8, 0.6]`, `[-0.8, 0.36, -0.48]`, and `[-0.6, -0.48, 0.64]` before
   doubling it. Gradient noise reads zero at every lattice point, and the turn
   keeps the octaves' lattices from lining up.
   [More noise](https://iquilezles.org/articles/morenoise/) explains the effect
4. The raw sum `s` has the standard deviation
   `sigma = sigma1 * sqrt(sum of the squared amplitudes)`. `sigma1` reads
   0.1816, the deviation of one layer measured over 20 million points spread
   evenly through the lattice. `fbm` returns the algebraic
   [sigmoid](https://iquilezles.org/articles/sigmoids/) `x / sqrt(1 + x * x)` of
   `x = 0.8 * s / sigma`. The sigmoid spreads the values about evenly across
   `(-1, 1)`

### Boxes

A shape's axis-aligned box holds every point where `d <= 0`. Where an axis runs
at an angle, the box comes from the
[3D boxes](https://iquilezles.org/articles/bboxes3d/) formulas.

1. A `sphere`, an `ellipsoid`, and an `octahedron` span their center plus and
   minus their radius or radii
2. A `box` and a `boxFrame` span their corners, and a `pyramid` spans its base
   square up to its apex
3. A `cylinder` and a `cone` take the box around their two end disks. An end of
   radius `r` spans its center plus and minus `r * sqrt(1 - n * n)` per axis,
   with `n` the unit axis
4. A `capsule` spans its ends plus and minus its radius, and a `roundCone` takes
   the box around its two end spheres
5. A cut or whole `torus` spans its center plus and minus
   `ringRadius * sqrt(1 - n * n) + tubeRadius` per axis
6. In the plane, a `circle` and an `ellipse` span their center plus and minus
   their radius or radii, and a `rect` and an `arch` span their corners. An
   `ngon`, a `star`, and a `polygon` take the box around their vertices, and a
   `polyline` takes the box around its points grown by half its width. A
   `spline` takes the box of the `polygon` or `polyline` its curve forms, and a
   `text` takes the box around its segments' end and control points
7. An `arc` takes the box around its end points and the points at `radius` in
   each axis direction its span crosses. Half its width then grows that box. A
   `sector`'s box also holds its center, and a `vesica` spans `a` and `b` grown
   by half its width
8. `union`, `mirror`, `repeat`, and `repeatPolar` take the box around their
   operands or copies. Each step of a `smoothUnion` fold grows the box around
   its two operands by `radius / 4`
9. `intersect` and `smoothIntersect` take the overlap of their bounded
   operands. `subtract` and `smoothSubtract` take the base's box
10. A transform takes the box around its child's eight transformed corners
11. `offset` grows the box by a positive distance and keeps it for a negative
    one. `shell` keeps it, `displace` grows it by `amplitude`, and `elongate`
    grows it by half its lengths on each side
12. `twist` takes the box around the cylinder about its axis line that holds the
    child's box. `bend` takes the box around the annulus sector the child's box
    bends into
13. `extrude` takes the profile's box between `from` and `to`. `revolve` takes
    the box around the circle the profile's largest u sweeps, across the
    profile's v span. `sweep` takes the box around a cube at each sample whose
    half side equals the sample's scale times the distance from the origin to
    the profile box's farthest corner
14. A `halfSpace` has no box, and neither does a shape built on one until an
    `intersect` with a bounded operand gives the result a box again

## Materials

```jsonc
{
  "materials": [
    // material: properties by name
    {
      "kind": "material",
      "properties": {
        "baseColor": "#8A5A2B",
        "roughness": 0.8,

        // custom properties
        "locked": true,
        "lootTier": { "kind": "int", "value": 3 },
        "loot": { "kind": "json", "value": { "coins": [1, 5] } },
      },
    },

    // shade: shade `index` of the `shades` entry `shades`; the darkest takes
    // index 0
    { "kind": "shade", "shades": 0, "index": 0 },
    { "kind": "shade", "shades": 0, "index": 1 },
    { "kind": "shade", "shades": 0, "index": 2 },
  ],

  "shades": [
    {
      // an index into `materials`
      "base": 0,

      "count": 3,

      // optional: the lightness step between neighboring shades; defaults to
      // 0.05
      "spread": 0.05,
    },
  ],
}
```

The eight named properties follow glTF's metallic-roughness model, except that
`metallic` defaults to 0:

| Property            | Value                    | Range             | Default   |
| ------------------- | ------------------------ | ----------------- | --------- |
| `baseColor`         | `#RRGGBB` or `#RRGGBBAA` | any               | `#FFFFFF` |
| `emissiveColor`     | `#RRGGBB`                | any               | `#000000` |
| `emissiveStrength`  | number                   | `[0, inf)`        | 1         |
| `ior`               | number                   | `0` or `[1, inf)` | 1.5       |
| `metallic`          | number                   | `[0, 1]`          | 0         |
| `occlusionStrength` | number                   | `[0, 1]`          | 1         |
| `roughness`         | number                   | `[0, 1]`          | 1         |
| `transmission`      | number                   | `[0, 1]`          | 0         |

1. A `#RRGGBB` color decodes as sRGB and passes through the sRGB transfer into
   linear light. A `baseColor` alpha stays straight and skips the transfer. A
   `baseColor` without alpha takes an alpha of 1
2. Any other name holds a custom property, whose kind follows its value: a
   boolean, a string, a number, or an array of 2 to 4 numbers.
   `{ "kind": "int", "value": ... }` holds a whole number or an array of 2 to 4
   of them, and `{ "kind": "json", "value": ... }` holds any JSON value. A
   material leaving out a custom property another material sets reads the
   kind's empty value there: 0, a zero vector, `false`, `""`, or `null`
3. `shades` steps the base color's
   [Oklab](https://bottosson.github.io/posts/oklab/) lightness. Shade `i` of
   `count` steps the lightness by `(i - (count - 1) / 2) * spread`. A step of 0
   keeps the base color. A step down scales the color in linear light toward
   black by the amount that reaches the stepped lightness. A step up keeps the
   color's Oklab hue and chroma at the stepped lightness. Every other property
   and the alpha carry over

## Patterns

A pattern picks one of its materials per point and never blends them.

```jsonc
[
  // bands: slabs `period` thick across `axis`, cycling through `materials`,
  // indices into `materials`
  {
    "kind": "bands",
    "materials": [0, 1],
    "axis": "y",
    "period": 0.1,

    // optional: bends the slabs by up to this much noise; defaults to 0
    "warp": 0.02,

    // optional: defaults to 0
    "seed": 7,
  },

  // grain: rings `period` thick around the line along `axis` through the
  // frame's origin, each a random pick. `warp` defaults to 0.75 * `period`,
  // and `seed` is required
  {
    "kind": "grain",
    "materials": [1, 2, 3],
    "axis": "y",
    "period": 0.05,
    "seed": 1,
  },

  // gradient: equal spans from `from` to `to` along `axis`, taking the
  // materials in order. `warp` and `seed` default to 0
  {
    "kind": "gradient",
    "materials": [0, 1, 2],
    "axis": "y",
    "from": 0,
    "to": 1,
  },

  // noise: value ranges of fractal noise with features about `scale` across,
  // taking the materials in order; `octaves` defaults to 4
  { "kind": "noise", "materials": [0, 1], "scale": 0.2, "seed": 3 },

  // cells: irregular cells about `size` across, each a random pick
  {
    "kind": "cells",
    "materials": [0, 1, 2],
    "size": 0.15,
    "seed": 5,

    // optional: a seam `width` wide between the cells, filled with
    // `material`; defaults to none
    "border": { "material": 3, "width": 0.02 },
  },

  // bricks: courses `height` tall along `axis` of bricks `length` long, each
  // a random pick; `axis` defaults to "y"
  {
    "kind": "bricks",
    "materials": [0, 1],
    "length": 0.215,
    "height": 0.065,
    "seed": 4,

    // optional: a seam `width` wide between the bricks, filled with
    // `material`; defaults to none
    "border": { "material": 3, "width": 0.01 },
  },

  // speckle: cubes `size` across that each take a random accent from
  // `accents` at `density` odds and `base` otherwise
  {
    "kind": "speckle",
    "base": 0,
    "accents": [1, 2],
    "size": 0.025,
    "density": 0.1,
    "seed": 9,
  },

  // checker: alternates its materials over cubes `size` across
  { "kind": "checker", "materials": [0, 1], "size": 0.1 },
]
```

### Pattern frames

Evaluating a shape at a point returns its distance and a frame position. The
frame position carries the point back through every transform and modifier to
the frame of the primitive that decides the distance.

1. A primitive returns the point it was evaluated at
2. A transform or modifier passes on its child's frame position. `twist` and
   `bend` evaluate the child at the unwarped point, and a pattern therefore
   follows the warp
3. `union` and `smoothUnion` pass on the frame position of the operand with the
   smallest distance among all their operands, and the earliest operand wins a
   tie. `intersect`, `subtract`, and their smooth variants pass on their first
   operand's frame position
4. `extrude`, `revolve`, and `sweep` pass on their 3D point from before the
   plane mapping

A `coat` or a `set` pattern reads the point itself.

### Pattern formulas

In the formulas below, `q` is the frame position, `n` counts the materials, and
`mod` takes the floored remainder. Every `fbm` and hash runs on the pattern's
seed, and a seed left out reads 0. Every `fbm` sums 4 octaves unless a `noise`
sets `octaves`. `hash(x, ...)` folds its whole-number arguments into the
[noise](#noise) hash in order. `unit(h)` computes `h / 2^32`.

1. `bands` computes `s = q[axis] + warp * fbm(q / (4 * period))`, and the point
   takes material `mod(floor(s / period), n)`
2. `grain` computes `s = r + warp * fbm(q / (4 * period))`, where `r` is the
   length of `q` with its `axis` component set to 0. The point takes material
   `mod(hash(floor(s / period)), n)`
3. `gradient` computes
   `t = (q[axis] + warp * fbm(q / ((to - from) / 4)) - from) / (to - from)`, and
   the point takes material `floor(t * n)` held within `[0, n - 1]`
4. `noise` computes `t = (fbm(q / scale) + 1) / 2`, and the point takes
   material `floor(t * n)` held within `[0, n - 1]`
5. `cells` works in lattice units of `size`. The lattice cube `c` holds one
   feature point at
   `c + [unit(hash(c, 0)), unit(hash(c, 1)), unit(hash(c, 2))]`
   - The search for the nearest feature point scans the 27 cubes around the
     point's cube in raster order with x outermost, and the first strict
     minimum wins. The nearest feature point's cube `c` gives the point
     material `mod(hash(c, 3), n)`
   - With `a` the offset from the point to the nearest feature point, the
     border's material takes the point when another feature point at offset
     `b` among the 125 cubes around the nearest one's cube gives
     `size * dot((a + b) / 2, (b - a) / length(b - a))` of half the border's
     `width` or less. The
     [Voronoi edges](https://iquilezles.org/articles/voronoilines/) article
     derives this distance
   - Each offset adds its cube's offset and its jitter before subtracting the
     point's position within the point's cube
6. `bricks` stacks courses along `axis`. The point's course reads
   `k = floor(q[axis] / height)`, and each other axis `i` reads
   `s[i] = q[i] / length - (1 + 2 * mod(k, 2)) / 4`
   - The brick `b` holds `k` on `axis` and `floor(s[i])` on each other axis.
     The point takes material `mod(hash(b[0], b[1], b[2]), n)`
   - On `axis`, `t = q[axis] / height` and `size = height`. On each other axis,
     `t = s[i]` and `size = length`
   - With `f = t - floor(t)`, the border's material takes the point when
     `size * f < width` on any axis, so each brick opens with its joint
7. `speckle` reads the point's sample position `x`: where the point sits in the
   frame a reader samples in. The cube `c = floor(x / size)` gives the point an
   accent when `unit(hash(c, 0))` reads below `density`. With `m` accents, the
   point takes accent `mod(hash(c, 1), m)`
8. `checker` gives the point material
   `mod(floor(q[0] / size) + floor(q[1] / size) + floor(q[2] / size), n)`

## Fonts

`fonts` holds the fonts a `text` reads. Each font takes a `name` that errors
report it by. Its `kind` picks where its data comes from:

```jsonc
[
  // file: a font file at a path, absolute or relative to the document's
  // directory. A font collection gives its first face
  { "kind": "file", "name": "label", "path": "../fonts/plex.otf" },

  // installed: the installed face whose family or full name matches `font`,
  // ignoring case. A family picks its regular face
  { "kind": "installed", "name": "sign", "font": "Futura" },
]
```

## Names

`names` holds a map per kind of entry a document can name. Each map takes a name
to an index. `shapes3d`, `shapes2d`, `materials`, `patterns`, and `steps` index
their tables, and `parts` indexes `nodes`. One entry can take several names.

```jsonc
{
  "names": {
    "shapes3d": { "leg": 2 },

    // two names for one material
    "materials": { "walnut": 0, "wood": 0 },

    "parts": { "stool": 3 },
  },
}
```

## Validation

Validation is a hard contract. A reader rejects any document that breaks a rule
below and never repairs, coerces, or fills in bad input.

1. `version` reads 1
2. Types are exact, and nothing is coerced. Every number is finite, and every
   index holds a whole number
3. `null` rejects everywhere except inside a `json` value
4. An unknown key rejects in every structure, and so does a `kind` the entry's
   table lacks. The only open points are a material's property names and
   `json` values. Keys are unique in every object, `json` values and `names`
   included
5. References:
   1. Every index points into its table
   2. An entry in `shapes3d`, `shapes2d`, or `nodes` references only earlier
      entries of its own table
   3. A shade's `index` stays below its `shades` entry's `count`, and the
      `shades` entry's `base` comes before the shade in `materials`
   4. `rootNodes` lists each node at most once, and a root node sits in no
      `childNodes`
   5. Every name in `names` points into its table
6. Arguments:
   1. Radii, widths, thicknesses, sizes, scales, periods, `spread`, and a
      `coat`'s `depth` are above zero, except that a `cone` end may take 0. A
      `round`, a `chamfer`, and a `sweep`'s `scales` are zero or more
   2. Counts and `octaves` are whole numbers above zero. An `ngon`'s `sides` is
      at least 3 and a `star`'s `points` at least 2. A seed is a whole number
      from `-2^31` to `2^31 - 1`
   3. A pattern's `materials` and a `speckle`'s `accents` hold at least one
      material. A `union`, an `intersect`, and their smooth variants hold at
      least one shape, and an `onto` holds at least one step. A `polygon` and a
      `spline` without `width` list at least 3 points, and a `polyline`, a
      `lathe`, a `spline` with `width`, and a `sweep` list at least 2. A
      `sweep` lists one scale per point. A `text` string holds at least one
      character and no line break
   4. `amplitude`, `warp`, and the lengths of an `elongate` are zero or more,
      `density` falls in `[0, 1]`, and a range's end passes its start
   5. Colors parse as `#RRGGBB`, or `#RRGGBBAA` for `baseColor`, and the other
      named properties fall in their ranges. A custom property takes a
      non-empty name and a value with a kind, and every material that sets it
      gives it one kind. An `int` holds whole numbers within `2^53 - 1`, and a
      vector holds 2 to 4 numbers
   6. Every shade's stepped lightness stays in `[0, 1]`, a step up stays inside
      linear sRGB at the color's chroma, and every shade stays inside `[0, 1]`
      in linear light
   7. The corners of a `box`, a `boxFrame`, and a `rect` differ on each axis. A
      `rect` takes `round` or `chamfer` but not both, and either stays within
      half its shorter side. A `box`'s `round` stays within half its shortest
      extent, and a `cylinder`'s within its radius and half its length
   8. The ends of a `cylinder`, a `cone`, a `capsule`, a `roundCone`, and a
      `vesica` differ, as do the neighboring points of a `polyline`, a
      `spline`, and a `sweep`. A `spline` without `width` takes its last point
      as the first point's neighbor. A `roundCone`'s radii differ by less than
      the distance between its ends
   9. A `star`'s inner radius stays below its outer radius, and an `arch`
      stands at least half its width tall. An `arc`, a `sector`, and a cut
      `torus` span at most 360 degrees. A `torus` takes both `from` and `to` or
      neither
   10. `orient` takes two directions of nonzero length that do not point
       opposite ways. An `elongate`'s `center` lies inside its shape's box
       along each axis it stretches. A `bend`'s `along` and `toward` lie on
       different axes, and its shape has a box that spans at most
       `pi * radius` along `along`
   11. A `polygon` outline and the curve of a `spline` without `width` never
       cross themselves
   12. A `sweep`'s `up` has nonzero length. The curve never runs along `up`:
       `cross(t, up)` turns less than a quarter turn between neighboring
       samples. Along the curve, the scale stays at 0 or more
   13. A `set` lists each point once
7. Steps and parts:
   1. A step that takes a material holds exactly one of `material` and
      `pattern`
   2. Every `add` shape has a box
   3. Every step has a non-empty name no other step in its object shares, and
      every part has a non-empty name no other child of its parent shares. The
      root parts count as one list
   4. Every step in an `onto` is an `add` earlier in each object holding the
      `paint` or the `coat`
8. Fonts, checked as a `text` reads its font:
   1. The font loads. A `file` font reads as a font, and an `installed` font
      matches an installed face
   2. The font holds every character of the string and gives a cap height or
      holds an `H`
   3. The text draws at least one contour

## Examples

### Chair

Four turned legs and a seat, each in grained oak:

```jsonc
{
  "version": 1,
  "shapes3d": [
    // 0: one leg's profile, turned about the y axis
    {
      "kind": "lathe",
      "points": [
        [0.0375, 0],
        [0.03, 0.15],
        [0.045, 0.225],
        [0.03, 0.425],
      ],
    },

    // 1: the leg moved to one corner
    { "kind": "translate", "shape": 0, "offset": [0.2, 0, 0.2] },

    // 2: the four legs
    { "kind": "mirror", "shape": 1, "axes": "xz" },

    // 3: the seat
    { "kind": "box", "min": [-0.25, 0.425, -0.25], "max": [0.25, 0.475, 0.25] },
  ],
  "materials": [
    // 0: oak, and 1 to 3: its shades
    {
      "kind": "material",
      "properties": { "baseColor": "#8A5A2B", "roughness": 0.8 },
    },
    { "kind": "shade", "shades": 0, "index": 0 },
    { "kind": "shade", "shades": 0, "index": 1 },
    { "kind": "shade", "shades": 0, "index": 2 },
  ],
  "shades": [{ "base": 0, "count": 3 }],
  "patterns": [
    // grain running up the legs and across the seat
    {
      "kind": "grain",
      "materials": [1, 2, 3],
      "axis": "y",
      "period": 0.05,
      "seed": 1,
    },
    {
      "kind": "grain",
      "materials": [1, 2, 3],
      "axis": "x",
      "period": 0.05,
      "seed": 2,
    },
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

### Forest

One tree stands in three places, and one rabbit sits under two of the trees:

```jsonc
{
  "version": 1,
  "shapes3d": [
    // 0: the trunk, 1: the crown, and 2: the rabbit's body
    { "kind": "cylinder", "a": [0, 0, 0], "b": [0, 0.5, 0], "radius": 0.05 },
    { "kind": "sphere", "center": [0, 0.65, 0], "radius": 0.25 },
    {
      "kind": "ellipsoid",
      "center": [0, 0.05, 0],
      "radii": [0.05, 0.05, 0.075],
    },
  ],
  "materials": [
    // 0: bark, 1: leaf, and 2: fur
    { "kind": "material", "properties": { "baseColor": "#4A3020" } },
    { "kind": "material", "properties": { "baseColor": "#3A7A2A" } },
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
    // 0 and 1: the parts every place shares
    { "name": "tree", "childObjects": [0], "childNodes": [] },
    { "name": "rabbit", "childObjects": [1], "childNodes": [] },

    // 2 and 3: offsets add up, so the rabbit under `tree.1` lands at
    // [-0.6, 0, 0.15]
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

    // 7: the root, holding the three trees
    { "name": "forest", "childObjects": [], "childNodes": [3, 4, 6] },
  ],
  "rootNodes": [7],
  "names": {
    "materials": { "bark": 0, "leaf": 1 },
  },
}
```
