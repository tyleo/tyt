# Model evaluation

_Part of the [voxel modeling plan](../../plan/open/voxel-modeling/README.md)._

This page sets how vxl builds a model file into an `.sdfj` document and how it
voxelizes that document into voxj. The
[sdfj format](../../../projects/sdf-formats/sdfj/docs/sdf-json-file-format.md)
sets what each entry computes at a point. The
[TypeScript API](../../../projects/utilities/sdfj-builder/docs/typescript-api.md)
lists the calls.

## Building

`vxl sdf-doc build` runs in order:

1. The flags and profile merge into the [settings](#settings)
2. vxl writes the builder's files and the [libraries](#libraries) and
   [fonts](#fonts) the settings list to a temporary directory
3. The runtime runs the builder on the model. It shares vxl's standard output
   and error
4. When the runtime exits with code 0, vxl copies the document to the output
   path. Any other exit fails the build and leaves the output path untouched

The [sdfj-builder README](../../../projects/utilities/sdfj-builder/README.md)
sets how the builder writes the document.

## Voxelizing

`vxl sdf-doc voxelize` runs in order:

1. The flags and profile merge into the [settings](#settings)
2. vxl reads the document and the
   [fonts](../../../projects/sdf-formats/sdfj/docs/sdf-json-file-format.md#fonts)
   its `text` shapes use, then runs the [checks](#checks)
3. The [sampling](#sampling) settles the voxel size
4. Each part's steps fill its [grid](#grids)
5. The fill mode empties the cells that do not show
6. vxl writes the [voxj document](#the-voxj-document). `--report` also prints
   the [report](#the-report) and writes it beside the document as
   `<stem>-report.txt`

## Settings

Both commands merge their flags and profile the way
[`vxl mesh-doc voxelize`](../../plan/open/vxl-commands/reference/mesh-doc/voxelize.md)
does. `--profile` reads `sdfDoc.build.profiles` or `sdfDoc.voxelize.profiles`
from the
[`.vxlconfig` cascade](../../plan/open/vxl-commands/reference/mesh-doc/voxelize.md#profiles).
vxl ships no profiles. When two files define one name, the later file wins.

A profile holds flags by their camel-case names, with the values the command
line takes:

| Profile  | Keys                                                                |
| -------- | ------------------------------------------------------------------- |
| build    | `runtime`, `intern`, `libraries`, `fonts`, `vars`, `varsFrom`       |
| voxelize | `resolution`, `voxelSize`, `frame`, `fillMode`, `flatten`, `report` |

`libraries` lists library names, `fonts` maps names to [fonts](#fonts), `vars`
maps names to JSON values, and `varsFrom` lists profile names. `resolution`
holds a `reference` and a `count`, and a voxelize profile sets `resolution` or
`voxelSize` but not both. Either profile can add a one-line `description`. An
unknown key or value errors when the profiles load.

A flag on the command line replaces the profile's value. Either size flag
replaces both of the profile's sizes. The output paths and voxelize's encoding
flags exist only on the command line, with the defaults `mesh-doc voxelize`
gives them.

Build variables land in this order, and a later variable replaces an earlier
one of the same name:

1. The profile's `varsFrom` profiles in list order, each after its own imports
2. The profile's `vars`
3. Each `--vars-from` profile, after its imports
4. Each `--var`

## Libraries

`sdfDoc.build.libraries` defines libraries in the `.vxlconfig` cascade, on top
of vxl's built-in layer:

```jsonc
{
  "sdfDoc": {
    "build": {
      "libraries": {
        // a path resolves against this file's directory
        "files": { "woods": { "path": "woods.sdfj" } },

        // a document inline
        "embedded": {
          "trim": {
            "description": "Brass trim",
            "document": {
              /* ... */
            },
          },
        },
      },
    },
  },
}
```

The last layer to define a name wins. One layer that defines a name in both
groups errors. The built-in layer embeds `materials`, which holds a material for
each name the modeling API lists.

vxl validates each library the settings list against the
[sdfj format](../../../projects/sdf-formats/sdfj/docs/sdf-json-file-format.md#validation).
The builder reads the libraries in list order, and a later library wins a name.

`lib` and `mat` copy an entry into the model the first time the model uses its
name. The copy brings every entry it references, and later uses reuse the copy.
A library part comes from a node. The node's object holds the part's steps, and
its child nodes are the child parts. A copied entry keeps its library names and
takes the name of each named export that holds it.

## Fonts

`--font <kind> <name=font>` sets the font a model's `text` reads by name, as in
`--font installed sign=Futura`. A build profile's `fonts` holds the same fonts:

```jsonc
{
  "fonts": {
    // a path resolves against this file's directory
    "label": { "kind": "file", "path": "fonts/plex.otf" },
    "sign": { "kind": "installed", "font": "Futura" },
  },
}
```

A `--font` path resolves against the working directory. A `--font` replaces the
profile's font of the same name. The document's
[`fonts`](../../../projects/sdf-formats/sdfj/docs/sdf-json-file-format.md#fonts)
entry holds each path relative to the document's directory.

## Sampling

`--voxel-size` sets the voxel size `g`, which defaults to 1. `--resolution`
instead divides a reference side by its count:

1. A world reference measures the box around every part's `add` shapes and
   `set` cubes at every place
2. An object reference measures the box around each part's own `add` shapes and
   `set` cubes and takes the extreme across parts, even under `--flatten`

Neither box rounds to the lattice. A part with no `add` or `set` has no box.

The lattice holds cubes `g` wide with corners on whole multiples of `g`. The
cell `[i, j, k]` has its center at `c = [i + 0.5, j + 0.5, k + 0.5] * g`. vxl
places each part on the lattice by a rigid map `P` and fills a cell when the
shape's distance at `P^-1(c)` reads zero or less. A place's offset `o` adds the
part's `offset` to the offsets of every part above it.

| Frame                                    | `P`                                                                       | Samples        |
| ---------------------------------------- | ------------------------------------------------------------------------- | -------------- |
| `--frame world`                          | moves the part by `o`                                                     | once per place |
| `--flatten nodes` or `--flatten objects` | also turns the part by each rotation up to the nearest joint or root part | once per place |
| `--frame local`                          | leaves the part where it is                                               | once per part  |

Under flattening, each rotation turns about its own part's pivot, and a `set`
cube turns with its part. A joint samples unturned.

## Arithmetic

All arithmetic runs in f64. Addition, subtraction, multiplication, division, and
the square root round the same on every platform. Sine, cosine, and arc tangent
can differ in the last bit.

vxl therefore computes these sines and cosines once, as it reads the document:

- rotations and `repeatPolar` copies
- the directions of `ngon` and `star`
- the ends of `arc`, `sector`, and a cut `torus`
- the middle of a `bend`

An angle first reduces into `[0, 360)` degrees. Multiples of 90 degrees give
exact zeros and ones. Other multiples of 30 and 45 degrees give exactly `1 / 2`,
`sqrt(2) / 2`, and `sqrt(3) / 2`. A quarter turn then keeps a box's corners on
the cell corners.

`twist` still takes a sine and a cosine per point, and `bend` an arc tangent. A
cell within rounding of a twisted or bent surface can land either way on
different platforms.

A few more rules keep results the same everywhere:

1. A length sums its squares in x, y, z order and takes the square root. It
   never calls a library `hypot`
2. Each sign comes from a comparison that says which side zero falls on. No
   formula calls a sign function
3. Rounding to the nearest whole number computes `floor(x + 0.5)`
4. A NaN stops the run as a bug, since the [checks](#checks) already reject
   every degenerate input

## Grids

Each part voxelizes into its own grid.

Once `g` settles, each 3D shape's
[box](../../../projects/sdf-formats/sdfj/docs/sdf-json-file-format.md#boxes)
moves by `P` and rounds out to whole cells. A shape's distance then reads at
least its distance to that rounded box. This clamp changes no sign, and it
leaves an exact distance exact. vxl skips a `mirror`, `repeat`, or `repeatPolar`
copy whose box lies farther than the nearest copy so far.

A part's grid spans the rounded boxes of its `add` shapes and the cells of its
`set` points. Cells outside the grid stay empty. `carve`, `paint`, and `coat`
only visit the grid's cells, so they can take an unbounded shape.

Each cell takes what the part's
[steps](../../../projects/sdf-formats/sdfj/docs/sdf-json-file-format.md#steps)
set at `P^-1(c)`. A `speckle` reads `c` as its sample position.

After the part's last step, `--fill-mode surface` empties each live cell that
does not show. A cell shows when a face neighbor is empty, or when the cell is
opaque and a face neighbor lets light through. A material lets light through
when any channel passes above zero at normal incidence. Cells outside the grid
count as empty.

## The voxj document

Each root part writes a root node at its pivot, in meters, scaled by `g` on
every axis. Below a root node, positions count voxels.

### Nodes

Each part writes one node named after it, listed by the node of every part that
holds it. A node takes its part's `rotation` and a scale of 1. Its position is
`(pivot - parentPivot) / g`, where each pivot is the part's `pivot` plus `o`.

A part in several places still writes one node. Each place has to sit the same
distance from its parent's pivot. Under `--frame world`, the places also have
to sit a whole number of voxels apart. voxelize errors otherwise.

`--flatten nodes` and `--flatten objects` write nodes only for the root parts
and joints, one per place. A joint's node nests under the node above it and
takes every rotation between the two.

### Objects

Each part writes one object holding the live cells of its first place, and
every place shares it. Under `--flatten nodes`, each place writes its own
object. Under `--flatten objects`, each root part and joint writes one object
for every part at or below it that no lower joint claims. Where two parts fill
one cell, the later place wins.

An object takes its part's name, and its `bounds` fit its cells. A part with no
live cell writes no object.

The node places its object by its `origin`. With `f = -(pivot + o) / g`, the
`origin` is the object's min cell index plus `f`. When `f` is not whole on every
axis, a child node named `voxels` sits at `f - floor(f)` and takes the object,
whose `origin` then adds `floor(f)` instead. Each voxel lands at its model
position, and turning the node turns the object about the pivot.

### Palette

Every object's one layer references one palette. The palette binds `baseColor`,
`metallic`, `roughness`, `emissiveColor`, `emissiveStrength`,
`occlusionStrength`, `ior`, and `transmission` in that order, then each custom
property by name. Each property gets its own value pool of distinct values.

Materials with identical properties merge. The palette lists its materials in
the order a raster scan first meets them, object by object, with x outermost.

### Order

Nodes and objects follow the order the places run in. A shared node and its
object come at the part's first place. vxl writes the document with the run's
`--format` and encoding flags and leaves out `editState` and `ext`.

## The report

The [modeling API](../../../projects/utilities/vxl/docs/modeling-api.md#report)
shows the report and what each field means. These rules pin the rest.

The report always reads grids sampled under `--frame world`, before the fill
mode thins them. Only the voxel counts read the thinned grids.

A part's steps print under its first place, and each later place's line reads
`repeats` with the first place's path. A lone root part takes no line, and its
steps and child parts follow the model's line.

A step's written cells are every cell an `add` or `set` filled, every live cell
a `carve` emptied, and every live cell a `paint` or `coat` recolored. `lost to`
and `over` list steps in list order and appear only when they hold one. A part
with live cells reads `detached` when its parent has live cells and no piece
holds both a parent cell and a cell of the part or a part below it.

Pieces number from the largest. Two pieces of one size order by which first
meets a raster scan with x outermost. A step name that repeats across lists
prints as its path of part names, leaving out a lone root part.

Meters round to six decimals with trailing zeros dropped, and `-0` prints as
`0`. A line over no cells leaves out its size and bounds. A count of one takes a
singular label, such as `1 cell`. Consecutive lines of one kind and depth pad
into columns with the counts right-aligned, and a column no line fills takes no
space.

## Checks

Each command stops at its first failed check. A voxelize check reports its step
by the step's path of part names. A step that writes no cell, or a part with no
live cell, shows in the report and stops nothing.

### Build

1. `--runtime` takes `node`, `bun`, or `deno`, `--font` takes `file` or
   `installed`, and `--profile` names a profile the cascade holds
2. `--library` and `libraries` name libraries the cascade defines, and each one
   passes the sdfj format's validation
3. A runtime that fails to start stops the build with the system's message

While the model runs, the builder makes the
[checks](../../../projects/utilities/sdfj-builder/README.md#errors) its README
lists. An error the model throws stops the build with its message.

### Voxelize

1. The flags take the values `vxl mesh-doc voxelize` lists. `--voxel-size` is
   above zero, and `--resolution` takes a whole count above zero
2. `--voxel-size` and `--resolution` exclude each other, and `--flatten nodes`
   and `--flatten objects` need `--frame world`
3. The document passes the
   [sdfj format's validation](../../../projects/sdf-formats/sdfj/docs/sdf-json-file-format.md#validation).
   A failing `shades` reports the largest spread that fits, rounded down to
   three decimals
4. A `--resolution` reference side is above zero
5. Each part's grid, and each root part's object under `--flatten objects`,
   holds at most 2^27 cells
6. Each `coat`'s `depth` reaches at least the voxel size
7. No distance reads NaN
8. The model holds at least one live cell
