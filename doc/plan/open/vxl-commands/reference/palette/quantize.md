# `vxl palette quantize`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl palette quantize <input> [output] --max-materials <n> [--index <palettes>]... [--property baseColor] [options]
```

Reduces each selected palette to at most `--max-materials` materials and
rewrites every layer that references it.

The command reads any voxel document and writes Voxel JSON. The candidates are
the materials some voxel samples, each weighted by its voxel count. A material
no voxel samples is dropped.

1. `--max-materials <n>` (required unless a profile sets it): the most materials
   the palette keeps.
2. `--index <palettes>` (default `*`): which palettes to quantize under the
   [palette selection](../../../../closed/palette-edit/README.md#palette-selection)
   rule. Each palette reduces separately. A palette no live voxel samples errors.
3. `--property <key>` (default `baseColor`): which property to cluster on. The
   palette has to bind it.
4. `--interpret-property` `auto` | `linear-color` | `srgb-color` | `numeric`
   (default `auto`): how the property's values read as points. `linear-color`
   and `srgb-color` read a vec-3-float or vec-4-float value as a linear or
   sRGB-encoded color and measure distance in `--space`. `numeric` reads any
   float, int, or vector value and measures Euclidean distance over its raw
   components. `auto` reads `baseColor` and `emissiveColor` as `linear-color`
   and any other numeric property as `numeric`, so a custom vector reads as a
   color only under an explicit color reading. The command errors on a bool,
   string, or json property, and on a color reading of a value other than a 3-
   or 4-float vector.
5. `--alpha` `partition` | `distance` | `ignore` (default `partition`): how a
   4-component color's alpha takes part. `partition` merges materials only when
   their alpha matches exactly. `distance` adds alpha as a fourth coordinate,
   scaled to the span of `--space`'s lightness axis. `ignore` leaves alpha out,
   so a merged voxel takes the representative's alpha. Passing the flag under
   any other reading errors.
6. `--partition <property>`: materials merge only when they agree on this
   property. `--partition metallic` keeps metals and dielectrics apart. The
   flag repeats, and `'*'` stands for every property other than `--property`.
   Every partition keeps at least one material, so the command errors when
   partitions outnumber `--max-materials`.
7. `--method` `median-cut` | `octree` | `kmeans` (default `median-cut`):
   clustering algorithm. `octree` needs 3D points and errors on a 4-component
   `numeric` value or under `--alpha distance`.
8. `--space` `oklab` | `lab` | `srgb` (default `oklab`): distance metric for a
   color reading. Passing the flag under `numeric` errors.
9. `--dither` `none` | `floyd-steinberg` | `ordered` (default `none`): error
   diffusion when snapping values, walking each object's voxels in 3D order, not
   a 2D image. Every referencing object dithers.
   [`object voxels quantize`](../object/voxels/quantize.md) takes the object
   selectors.
10. `--profile <profile>`: apply a saved reduction recipe. See
    [Profiles](#profiles).

Clustering runs on `--property`. Each cluster collapses onto a representative,
its most-sampled material. Every merged voxel takes all of the representative's
values. The representative is always a real material, never an average.

Surviving materials compact, which shifts material indices. The compaction also
prunes the value-pool values only dropped materials referenced.

The command runs [`object voxels quantize`](../object/voxels/quantize.md) with
`--shared` over every layer referencing the palette, then compacts. See
[Palettes](../../../../../../projects/voxel/formats/voxj/docs/voxel-json-file-format.md#palettes).

## Profiles

A profile saves a reduction recipe under a name in a `.vxlconfig`, at
`palette.quantize.profiles`. The files cascade as they do for
[`object mesh` profiles](../../../../../ref/mesh/profile-language.md#loading),
with no built-ins beneath them. `vxl profile palette quantize list` prints the
merged set grouped by the file supplying each name.

```jsonc
{
  "palette": {
    "quantize": {
      "profiles": {
        "retro-16": {
          "maxMaterials": 16,
          "dither": "ordered",
          "partition": ["metallic", "emissiveStrength"],
        },
        "roughness-8": {
          "maxMaterials": 8,
          "property": "roughness",
          "interpretProperty": "numeric",
        },
      },
    },
  },
}
```

A profile holds the reduction flags by camel-case name with their command-line
values: `maxMaterials`, `property`, `interpretProperty`, `alpha`, `partition`
as an array, `method`, `space`, and `dither`. `--index` stays on the command
line because it picks the run's target. An unknown key or value errors when the
profiles load.

A run applies at most one profile. A flag on the command line overrides the
value the profile sets, and `--partition` replaces the profile's whole list.

```
vxl palette quantize model.voxj
  --profile retro-16
  --dither none
```
