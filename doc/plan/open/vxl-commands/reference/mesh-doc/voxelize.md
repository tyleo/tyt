# `vxl mesh-doc voxelize`

*Part of the [Vxl Command-Line Reference](../../README.md).*

```
vxl mesh-doc voxelize <input> [output] [--resolution <reference> <n> | --voxel-size <meters>] [--frame <frame>] [--scale <scale>] [options]
```

Rasterizes a mesh into voxel objects. This is the inverse of
[`vxl object mesh`](../../../../../ref/mesh/mesh.md). The input is a glTF mesh,
text (`.gltf`) or binary (`.glb`); glTF is the only mesh format read for now.
The default output path is the input stem with the `.voxj` extension. The voxel
size is set one of two mutually exclusive ways: a voxel count along a reference
side with `--resolution` or the size directly with `--voxel-size`. When neither
is given it defaults to `--voxel-size 1`, one voxel per meter.

Every mesh object the hierarchy places becomes one voxel object under a root
node named after its placing mesh node. Each object's grid sits on a lattice
of voxel-size cubes anchored at the origin of its frame, and the object's
lattice cell is recorded as its origin. `--frame` and `--scale` choose the
frame. The voxel object is named after the mesh object, else its placing node,
else the input file stem. The objects share one palette holding every distinct
sampled material. [`palette quantize`](../palette/quantize.md) can reduce the
palette afterward. An object with no triangles is an error that reports the
object.

1. `--from` `gltf` | `glb`: source mesh format, glTF text or binary. Inferred
   from the input extension when omitted.
2. `--resolution <reference> <n>`: divide a reference side into `<n>` voxels.
   The voxel size is that side over `<n>`, and every other side takes as many
   voxels as cover it. World references measure the bounds of every object
   together: `longest-world`, `shortest-world`, and `world-x` | `world-y` |
   `world-z`. Object references measure each object's bounds and take the
   extreme across objects: `longest-object` and `shortest-object` over any
   side, and `longest-object-x` | `longest-object-y` | `longest-object-z` and
   `shortest-object-x` | `shortest-object-y` | `shortest-object-z` along one
   axis. A shortest reference skips sides with no extent. A reference with no
   extent, such as `world-y` on a flat mesh, is an error. Use this to cap
   detail at a known voxel count.
3. `--voxel-size <meters>` (default `1`): the edge length of one voxel in meters.
   Each axis takes as many voxels as cover the mesh extent there, so the same
   `<meters>` yields a consistent real-world voxel size across meshes of
   different sizes. Mutually exclusive with `--resolution`, and used with
   `<meters>` of `1` when neither flag is given. The format carries no physical
   units: one unit is one voxel, and real-world scale comes from hierarchy-node
   transforms. Both flags resolve to one voxel size, which `mesh-doc voxelize`
   records in the placing node's scale so the assembled model keeps its source
   dimensions. glTF is meter-native, and under `--scale bake` any scene- or
   node-level scale on the mesh is applied before voxelizing, so two glTF
   exports of the same object at different authored scales voxelize alike,
   mirroring [`vxl object mesh`](../../../../../ref/mesh/mesh.md)'s
   `--voxel-size`. See
   [Coordinate System](../../../../../../projects/voxel/formats/voxj/docs/voxel-json-file-format.md#coordinate-system).
4. `--frame` `world` | `local` (default `world`): the frame each object's grid
   is built in. `world` applies the placing node's rotation and translation to
   the geometry, so every object's voxels align on one lattice and an object
   two nodes place voxelizes twice. `local` builds the grid in the object's
   own space and keeps the rotation and translation on the node, so an object
   two nodes place voxelizes once and both nodes share it.
5. `--scale` `bake` | `keep` (default `bake`): what happens to a placing
   node's scale. `bake` applies it to the geometry, so every object's cubes
   are the voxel size. `keep` leaves it on the node, so a scaled object's
   cubes are the voxel size times its scale, and the voxel size and object
   references are in unscaled units. World references are rejected under
   `keep`, which has no world voxel size. The four combinations:

   |                 | `--scale bake`                                     | `--scale keep`                                           |
   | --------------- | -------------------------------------------------- | -------------------------------------------------------- |
   | `--frame world` | one aligned lattice, every cube the same size      | one aligned lattice, a scaled object's cubes scaled      |
   | `--frame local` | each object its own grid, every cube the same size | each object its own grid, a scaled object's cubes scaled |

   Under `--frame local`, `bake` shares a repeated object only between
   placements at the same scale, since the scale sets its grid; `keep` shares
   it across every placement.
6. `--flatten` `none` | `nodes` | `objects` (default `none`): how much of the
   hierarchy the output flattens. `none` writes a root node for each placed
   object. `nodes` writes a root node for each mesh root node. That node takes
   the mesh root node's name and holds every object placed at or below the mesh
   root node on one grid. `objects` writes one object there instead. The object
   takes the mesh root node's name, else the input file stem. Where two
   placements cover one voxel, the one placed later wins. Flattening needs
   `--frame world` and `--scale bake`.
7. `--fill-mode` `solid` | `surface` (default `solid`): how the mesh fills the
   grid. `solid` rasterizes the surface and flood-fills the volume it encloses,
   producing a filled body, and expects a watertight mesh. `surface` rasterizes
   only the voxels the triangles pass through, leaving a hollow shell.
8. `--material-mode` `auto` | `per-primitive` | `per-texel` | `flat` (default
   `auto`): where each voxel's color and material come from. `--fill-mode` sets
   the geometry; this sets the color, the two are independent.
   1. `per-primitive` reads each mesh material's flat factors (`baseColorFactor`,
      `metallicFactor`, `roughnessFactor`, `emissiveFactor`, `emissiveStrength`,
      `occlusionStrength`), giving one material per mesh material, so an untextured
      or stylized mesh stays exact with a tiny palette.
   2. `per-texel` samples those maps at each voxel's surface point, area-averaged
      over the voxel's footprint rather than point-sampled so fine texture does
      not alias into a muddy palette, capturing spatial detail at the cost of a
      larger palette.
   3. `flat` reads nothing from the mesh and paints the one `--fill-color`.
   4. `auto`, the default, picks `per-texel` when the mesh carries textures and
      `per-primitive` when it does not.

   Every mode writes the same properties
   [`object mesh`](../../../../../ref/mesh/mesh.md) bakes back, `baseColor`,
   `metallic`, `roughness`, `emissiveColor`, `emissiveStrength`, and
   `occlusionStrength`, so a voxelized model round-trips through `object mesh`.
9. `--fill-color <#RRGGBBAA>`: the color of voxels that have no sampled surface,
   omitted for the default. Its role depends on `--material-mode`:

   |                            | `--fill-color` omitted                        | `--fill-color #RRGGBBAA`             |
   | -------------------------- | --------------------------------------------- | ------------------------------------ |
   | `flat`                     | whole object white                            | whole object that color              |
   | `per-primitive`/`per-texel`| exterior sampled, interior its nearest surface | exterior sampled, interior that color |

   Only the interior voxels a `--fill-mode solid` body invents have no surface; a
   hollow `--fill-mode surface` shell is all surface, so under the sampling modes
   a set `--fill-color` is rejected there.
10. `--profile <profile>`: apply a saved voxelize recipe. See
    [Profiles](#profiles).

`mesh-doc voxelize` writes a voxel-json document and shares `vox-doc to voxj`'s
encoding options: `--format`, `--encoding-preset`, `--position-encoding`, and
`--sample-encoding`, which default the same way they do there. It does not take
`--ext` or `--edit-state`: a voxelized mesh has no source `ext` block to carry
and no editor build volume to record.

## Profiles

A profile saves a voxelize recipe under a name in a `.vxlconfig`, at
`meshDoc.voxelize.profiles`. The files cascade as they do for
[`object mesh` profiles](../../../../../ref/mesh/profile-language.md#loading),
with no built-ins beneath them. `vxl profile mesh-doc voxelize list` prints the
merged set grouped by the file supplying each name.

```jsonc
{
  "meshDoc": {
    "voxelize": {
      "profiles": {
        "coarse-flat": {
          "resolution": { "reference": "longest-world", "count": 16 },
          "materialMode": "flat",
          "fillColor": "#808080",
        },
        "props": {
          "voxelSize": 0.0625,
          "frame": "local",
          "outOfRangeProperty": "clamp",
        },
      },
    },
  },
}
```

A profile holds the voxelize flags by camel-case name with their command-line
values: `resolution` as an object of `reference` and `count`, `voxelSize`,
`frame`, `scale`, `flatten`, `fillMode`, `surfaceMode`, `materialMode`,
`fillColor`, and `outOfRangeProperty`. A profile sets `resolution` or
`voxelSize`, not both. `--from`, the output, and the encoding options stay on
the command line because they describe the files. An unknown key or value errors
when the profiles load.

A run applies at most one profile. A flag on the command line overrides the
value the profile sets. Either `--resolution` or `--voxel-size` replaces the
profile's `resolution` and `voxelSize` alike. A profile's values meet the same
checks as the flags, so a profile's world `resolution` under `--scale keep`
errors.

```
vxl mesh-doc voxelize model.glb
  --profile props
  --frame world
```
