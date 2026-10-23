# Voxel modeling

A voxel model starts as a TypeScript model file of shapes and materials in
meters. Each pass builds the model file, voxelizes it with a report, and renders
a review sheet. The TypeScript API below lists every call a model file can
make.

## A pass

```sh
vxl sdf-doc build table.ts --library materials && vxl sdf-doc voxelize table.sdfj --voxel-size 0.025 --report && vxl object render table.voxj --profile review --to png
```

The pass writes `table.sdfj`, `table.voxj`, `table-report.txt`, and the review
sheet `table.png` beside the model file. The sheet holds the `hero`, `front`,
and `right` views in its top row and the `back` and `top` views below them. A
voxelize without the build reads a stale `.sdfj` document. Every change goes in
the model file because the next pass overwrites the `.voxj`.

## Building a model

The smallest feature sets the voxel size. Every feature the prompt names spans
at least 2 voxels. A birdcage's 6 mm bars take voxels of about 3 mm, and the 60
cm cage then stands about 200 voxels tall. Models several hundred voxels across
voxelize in minutes. A large scene under a voxel budget keeps its real size with
`--fill-mode surface` or a coarser voxel.

The `hero` view looks from the front-right-top. The detail the prompt is about
faces +z or +x with nothing between it and that corner. A room or a hull keeps
its walls and takes a view from [other views](#other-views).

A first pass of boxes settles the proportions cheaply. Shapes, openings, small
details, and patterns then replace the boxes.

## Choosing operations

The API already does much of what a model would build by hand:

1. Gems on a pile, snow on a roof, and gilt on top faces take `coat` with
   `sides`. The coat finds the surface without a guessed height
2. Arms, spokes, and candles around a center take `repeatPolar`. Limbs, horns,
   and curled tips take `bend` or `twist`
3. A flame's core inside its outer flame takes one shape with a `gradient`
   because nested shapes bury each other
4. A `coat` or a `paint` recolors every live cell in its reach, a neighbor's
   included. `onto` keeps it on the cells of the listed `add` steps
5. Repeated props place one part at each spot

## Checks

The report reads more reliably than the PNGs:

1. The bounds on the model's line match the prompt's real size in meters. They
   read each posed part unrotated
2. A step at `0 cells` missed every cell center, and a step at `0 kept` lost
   every cell to the steps after `lost to`
3. A detail at `0 exposed` sits buried inside another form. A see-through
   detail inside glass or water reads `0 exposed` and still shows
4. A step reading more pieces than the forms it draws broke on the grid or under
   a later cut. A `paint` or a `coat` spilled when it reads `over` a step it
   should miss
5. A second piece means a shape floats, and a part reading `detached` has a gap
   at its joint. Forms meant to stay apart, such as chain links, need a cell of
   clearance or they fuse into one piece

`front`, `right`, `back`, and `top` draw without perspective and show heights,
widths, symmetry, and alignment true to scale. `hero` shows whether the model
reads as the prompt. Color goes wrong in a few known ways:

1. A large face of one flat color reads unfinished. `shades`, `noise`, or a
   pattern breaks the face up, and `bands` or `gradient` give a terrain's cut
   sides strata
2. Neighboring materials need colors far apart in lightness or hue. Two dark
   tones or two metals side by side merge
3. A thin pane of glass or sheet of water reads faintly. A darker rim outlines
   clear glass, and tinted glass tints what sits behind it
4. A tank-deep body of water reads dark and hides what sits in it, and clear
   glass over a dark backdrop hazes it pale. A low `baseColor` alpha such as
   `#B8E6F030` keeps either faint
5. A glow that reads pale where the prompt wants color takes a lower
   `emissiveStrength` or a darker `baseColor`

The model is done when every view shows the prompt with no flaw a review can
name and every extra piece and `0 kept` step in the report is intended.

## Other views

A close-up, another angle, or a view inside a room comes from flags on the
render command:

```sh
# table.png with its hero view from behind and to the left
vxl object render table.voxj
  --profile review
  --view-orbit hero 225 20 fit
  --to png

# robot-close.png with the head part alone
vxl object render robot.voxj
  --select 'robot/head'
  --view-orbit close 30 20 fit
  --to png
  --file-stem robot-close

# room-inside.png from eye height toward the fireplace
vxl object render room.voxj
  --view-frame inside world
  --view-position inside 1.2 1.6 1.4
  --view-look-at inside -0.3 0.9 -1.4
  --view-fov inside 60
  --to png
  --file-stem room-inside
```

1. `--view-orbit` places a view by an azimuth and an elevation in degrees and a
   distance in meters or `fit`. `--view-look-at` beside it sets a point to
   circle at a distance in meters
2. A new view such as `close` adds `table-close.png` and renames the sheet
   `table-review.png`. A close-up alone leaves `--profile review` off
3. A view placed by position takes `--view-frame`, `--view-position`, and a
   rotation flag such as `--view-look-at`
4. `--select` renders the matched parts alone. `--view-select <view> <select>`
   keeps the whole model and frames the view on the matched parts
5. `--rotate <part> <axis> <degrees>` rotates a part about its pivot for one
   render. A raised arm then shows its clearance. A pose the model keeps goes in
   the part's `rotation`

## Exporting a mesh

A prompt that asks for a mesh, a glTF, or a `.glb` file gets one from the
`.voxj` document after the last pass:

```sh
# table.glb with the materials baked into textures
vxl object mesh table.voxj
  --profile pbr

# table.glb with the gems, ice, and water kept transparent
vxl object mesh table.voxj
  --profile glass
```

1. `glass` replaces `pbr` for a model with a transparent material. The gems,
   `ice`, `water`, and any material with `transmission` or a `baseColor` alpha
   below 1 are transparent
2. `--to gltf` writes a text `table.gltf` with the textures embedded

The mesh keeps the model's size in meters and its parts as nodes.
