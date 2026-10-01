# vxl

A command-line tool for working with voxels.

## Editing

The `object`, `node`, and `palette quantize` commands edit a document. Each
reads any format voxconv reads and writes Voxel JSON beside the input by default.
`object mesh` writes a mesh instead. `node list` only prints the scene graph.
`--select` takes a hierarchy-path glob. `--select-index` takes an index or a
range. Both repeat and pick what the command acts on. `--select-parent` and
`--select-parent-index` pick the one node at the parent end of an edge.

```sh
# Reads Voxel Max and writes scene.voxj beside it.
vxl object remove scene.vmax --select 'debris/**'
```

## Objects

`object` commands write object properties and move the voxels along when needed.
`set origin` moves an object's grid within its node. `set edit-bounds` and
`trim` resize the grid without moving a voxel in the scene. `link` and `unlink`
add or drop one placement. `duplicate` copies objects within the document. `add`
copies objects from another file along with their palettes. `downsample` and
`upsample` change the grid's resolution by a whole-number factor per axis,
scaling `origin` and `bounds` with it, so `node set scale` keeps the object's
size in the scene. `mesh` writes the selected objects as a glTF mesh. The
[mesh reference](../../../doc/ref/mesh/mesh.md) covers its flags.

`render` draws the selected objects into one image per view, placed as `mesh`
places them. The views and lights come from profiles in `.vxlconfig` and the
flags that mirror them. With neither, the `hero` view renders under the
`studio` lights. `--profile glow` adds a bloom over the emissive
materials. The image shows inline in the terminal, or `--to png` writes
one PNG per view beside the input. The
[render reference](../../../doc/ref/render/render.md) covers the flags and the
profiles.

```sh
# Copies the first three props under the one node matching house.
vxl object add scene.voxj --source props.voxj --select-index 0-2 --select-parent house

# Grows the crate's edit box from [-2 -6 0]..[2 2 4] by 2 voxels on -x and -z.
vxl object set edit-bounds scene.voxj --select crate --min -4 -6 -2 --max 2 2 4

# Merges the crate's voxels two per axis, keeping a block at least half live.
vxl object downsample scene.voxj --select crate --factor 2

# Splits each of the crate's voxels into ten per axis.
vxl object upsample scene.voxj --select crate --factor 10

# Shows the crate from the front-right-top under the studio lights.
vxl object render scene.voxj --select crate

# Writes scene-front.png and scene-top.png beside the input.
vxl object render scene.voxj --profile front --profile top --to png
```

## Object Voxels

`object voxels` commands edit voxels within the grid and never change `origin`
or `bounds`. `translate` shifts the voxels, `flip` mirrors them, and `rotate`
turns them in quarter turns that follow the right-hand rule. One or three turns
need the two turned dimensions to be equal. `node set rotation` turns any
object.

`quantize` rewrites the materials voxels sample so each quantized layer samples
at most `--max-materials` materials of its palette. The palettes stay as they
are. Each object clusters apart unless `--shared` clusters the whole selection
together. The clustering flags match `palette quantize`'s.

```sh
# Turns the 6 x 8 x 6 crate a quarter turn about y.
vxl object voxels rotate scene.voxj --select crate --axis y --turns 1

# Brings the crate down to 16 materials, keeping metals and dielectrics apart.
vxl object voxels quantize scene.voxj --select crate --max-materials 16 --partition metallic
```

## Palettes

`palette list` and `palette show` print palettes. `palette quantize` reduces a
palette to at most `--max-materials` materials and snaps every voxel sampling
it. Each cluster collapses onto its most-sampled material, and a merged voxel
takes that whole material. Clustering runs on `--property`, `baseColor` by
default. `--partition` keeps materials apart unless they agree on a property.
Materials no voxel samples drop. Both quantize commands take `--profile`, which
applies flags saved in a `.vxlconfig` under `palette.quantize.profiles` or
`object.voxels.quantize.profiles`. `vxl profile palette quantize list` and
`vxl profile object voxels quantize list` print them.

```sh
# Reduces the first palette to Voxel Max's 255 colors.
vxl palette quantize scene.voxj --max-materials 255
```

## Nodes

`node list` prints the scene graph. The other `node` commands write nodes and
hierarchy edges. `set` writes a node's name, position, rotation, or scale.
`set rotation` takes Euler angles in the order `node list --show-transforms`
prints them, in degrees by default. `add` creates an empty node. `link` and
`unlink` add or drop one child edge. Without a parent selector, `add`, `link`,
and `unlink` act on the root list. `remove` releases nodes along with each
descendant left without a parent.

```sh
# Swings the door node 37 degrees about y.
vxl node set rotation scene.voxj --select house/door --rotation 0 37 0

# Places the same door node under garage too.
vxl node link scene.voxj --select house/door --select-parent garage
```
