# `vxl mesh`

_Part of the [Vxl Command-Line Reference](../README.md)._

_Superseded: the mesh plan is moving to [`doc/plan/open/mesh`](../../mesh/),
the [value language](../../mesh/value-language.md) and
[profile language](../../mesh/profile-language.md) subplans with their
[open questions](../../mesh/open-questions.md).
[Orphaned options](../../mesh/orphaned-options.md) lists what this page
holds that the new plan has yet to absorb._

```
vxl mesh <input> [output] [options]
```

Triangulates one object's voxels into a glTF mesh. It can also bake the
object's palette materials into textures the mesh's UVs sample. The default
output path is the input stem with the mesh extension. The format comes from
`--to`, else the output extension, else `.glb`.

```
vxl mesh turret.voxj                  # turret.glb, geometry only
vxl mesh turret.voxj --texture pbr    # + embedded albedo, orm, and emissive maps
```

`mesh` writes one object as pure geometry, with no hierarchy-node transform
applied. The common case is pulling a leaf object out without placement.
`--select` and `--select-index` choose the object; the selection must resolve
to exactly one, so a multi-object document needs a selector. See
[Object selectors](conventions.md#object-selectors). Assembling a placed scene
from the hierarchy is a separate [deferred](#deferred) mode.

1. `--to` `gltf` | `glb`: target mesh format, glTF text (`.gltf`) or binary
   (`.glb`). Inferred from the output extension when omitted, defaulting to
   `.glb`.
2. `--from <format>`: source voxel format. Inferred from the input extension
   when omitted.
3. `--voxel-size <meters>` (default `1.0`): the real-world edge length of one
   voxel in meters, the mesh twin of `voxelize`'s `--voxel-size`. The voxel
   grid is unitless and glTF is meter-native, so this is where a voxel gains a
   physical size: `1.0` opens at one meter per voxel, `0.01` at one
   centimeter. Applied as a uniform scale to vertex positions only.
4. `--method` `greedy` | `culled` | `naive` (default `greedy`): meshing
   strategy. `greedy` merges coplanar, same-material faces into the fewest
   quads. `culled` emits one quad per solid-empty boundary face, unmerged.
   `naive` emits all six faces of every solid voxel, hidden interior faces
   included. Choose `culled` or `naive` only when you need stable per-voxel
   topology.
5. `--atlas` `palette` (default `palette`): material-map atlas layout.
   `palette` is the only layout for now: one texel per flattened material the
   mesh uses. See [The palette atlas](#the-palette-atlas). An `unwrap` layout
   is [deferred](#deferred).
6. `--texture-shape` `line` | `fit` | `square` | `pot` | `<n>` (default
   `pot`): the atlas canvas. `line` is a single row of texels, `fit` the
   near-square packing, `square` the smallest square, `pot` the smallest
   square power of two, and `<n>` an exact `n`x`n` canvas, rejected when too
   small. Unused cells are transparent black the mesh never samples.
7. `--texture <preset>`: bake a preset material map. Repeatable, as
   `--texture albedo --texture orm`. The presets:
   1. `albedo`: RGBA base color from `baseColor`.
   2. `orm`: R = `occlusionStrength`, G = `roughness`,
      B = `metallic`.
   3. `metallic-roughness`: R = `0`, G = `roughness`,
      B = `metallic`.
   4. `metallic-smoothness`: R = `metallic`, A = smoothness
      (`1-roughness`), G and B = `0`.
   5. `mse`: R = `metallic`, G = smoothness, B = `emissiveStrength`.
   6. `emissive`: `emissiveColor` scaled by `emissiveStrength`, so a surface
      glows in its own color. The strongest material's strength is written as
      `KHR_materials_emissive_strength`.
   7. `occlusion`: grayscale `occlusionStrength`.
   8. `roughness`: grayscale `roughness`.
   9. `smoothness`: grayscale `1-roughness`.

   The one bundle, `pbr`, expands to `albedo`, `orm`, and `emissive`.
8. `--texture-name <preset> <file-name>`: name one preset's file exactly, as
   `--texture-name albedo skin.png`. Repeatable. The preset must be a
   single-map preset, must be baked, and may be named once.
9. `--texture-name-prefix <file-name>`: replace the default `<output-stem>-`
   prefix on every preset file `--texture-name` does not name. The preset
   follows the prefix verbatim, so the prefix carries its own separator:
   `hero-` writes `hero-albedo.png`, `test.` writes `test.albedo.png`.
10. `--texture-map <file-name> <channels>`: bake a custom packing. Repeatable.
    See [Channel expressions](#channel-expressions).
11. `--define-property <property> <name>`: name a custom voxel-json property
    so a custom packing can read it. Repeatable. See
    [Channel expressions](#channel-expressions).
12. `--texture-storage` `embedded` | `external` | `both`: where the baked
    images go. `embedded` packs them into the mesh, a `.glb` binary chunk or a
    `.gltf` data URI. `external` writes loose `.png` files beside the mesh.
    `both` embeds the copy the mesh references and writes the loose files as
    working copies. Defaults to `embedded` for `.glb` and `external` for
    `.gltf`.
13. `--select <glob>`: choose the object by hierarchy path, matched the way
    `hierarchy show` matches node paths, so a node path selects its subtree.
    Repeatable; unions with `--select-index`. See
    [Object selectors](conventions.md#object-selectors).
14. `--select-index <index>`: choose the object by position, an integer or an
    `a-b` range. Repeatable; unions with `--select`.

Every map file name is written beside the mesh. A name or prefix that is
empty or holds a path separator errors, and so do two maps resolving to the
same file name.

## The palette atlas

All the maps of one bake share a single atlas layout: one texel per distinct
flattened material. The object's layers merge per property name by the
format's layer-override resolution, each property read through the last layer
whose palette supplies its name, so a voxel's texel is keyed by the tuple of
materials it samples in those winning layers, deduplicated in first-seen
raster order. A single-layer object reduces to one texel per material its
voxels use. Each map fills the same layout from its own properties, so

```
vxl mesh turret.voxj --to gltf --texture albedo --texture orm
```

writes `turret.gltf`, `turret-albedo.png`, and `turret-orm.png`, the two
images the same size with the same flattened material at the same texel.
Every face's UVs sit at its texel center, read with a nearest-neighbor
sampler and clamped wrapping, so a face samples exactly its texel. The atlas
depends on the materials the object uses, so it is per-mesh, not shared
across meshes.

A property a material leaves unset takes its glTF spec default, so a map
never fails on a missing property. Once maps are baked, greedy meshing merges
only faces that share a flattened material, since a merged quad samples one
texel; pure geometry merges on shape alone.

A preset with a standard glTF slot binds it: `albedo` to `baseColorTexture`,
`metallic-roughness` to `metallicRoughnessTexture`, `orm` to both
`occlusionTexture` and `metallicRoughnessTexture` sharing one image,
`occlusion` to `occlusionTexture`, and `emissive` to `emissiveTexture`. The
slotless presets (`mse`, `metallic-smoothness`, `roughness`, `smoothness`)
and every `--texture-map` packing are instead listed by name under the
material's `extras.vxl.maps`, where a generic viewer ignores them and a
custom pipeline finds them.

## Channel expressions

`--texture-map <file-name> <channels>` bakes a custom packing. `channels` is
one argument: a comma-separated list over `R=<expr>`, `G=<expr>`, `B=<expr>`,
and `A=<expr>`. This reproduces the `mse` preset:

```
vxl mesh model.voxj --texture-map model-mse.png R=metallic,G=1-roughness,B=emissiveStrength
```

The image's channel count is the highest channel named; an unnamed channel is
`0`. Each `<expr>` is one of:

1. `<property>`: a scalar property by its voxel-json key, as
   `metallic`.
2. `<property>.<component>`: one component of a vector property through
   either alias set (`.r`/`.g`/`.b`/`.a` or `.x`/`.y`/`.z`/`.w`), as
   `baseColor.r`.
3. `1-<property>`: the inverse, as `1-roughness`.
4. `0` | `1`: a constant.

A property's type is never declared on the command line. A glTF vocabulary
name has the vocabulary's type whatever a palette binds it to; a custom key
takes its type from the shape of the value pool its key binds in its winning
layer's palette, the last layer whose palette supplies the name: a float
vector is a color, read one component at a time, and a `float`, `int`, or
`bool` pool is a scalar, read whole and rejecting a component. A key no layer
supplies follows the format's
[unbound-default rule](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#properties):
a glTF built-in bakes its spec default, a custom key errors.

`--define-property <property> <name>` gives a custom voxel-json key a name a
packing can read: `property` is the token used in `<channels>`, `name` is the
voxel-json key. The key is its own argument, so it may be quoted, and a key
with spaces is reachable only through an alias. A binding shadows a built-in
of the same name within the custom packings; the `--texture` presets always
read the spec properties. With a palette binding `tint` to a `vec-4-float`
pool,

```
vxl mesh model.voxj --define-property tint tint --texture-map paint.png R=tint.r,G=tint.g,B=tint.b,A=baseColor.a
```

packs the custom `tint` color into RGB and the base color's alpha into `A`.

## Deferred

Designed but unbuilt. The hidden CLI values (`--atlas unwrap`,
`--texture computed-occlusion`) parse but error until their features land. A
proposed redesign of the map flags around an expression language lives in the
[mesh value language](../../mesh/value-language.md) subplan.

### Vertex attribute maps

The texture flags' vertex twins: the same resolved values written to glTF
vertex attributes, one value per face corner, with no texture and no UV set.
Only `COLOR_0` is portable glTF. The rest are application-specific `_NAME`
attributes that only a custom shader reads.

1. `--vertex <preset>`: the texture presets, packed the same way. `albedo`
   writes `COLOR_0`. `computed-occlusion` multiplies into `COLOR_0` to darken
   it. The scalar presets write `_METALLIC`, `_ROUGHNESS`, and so on; the
   packed presets write `_ORM`, `_MSE`, and so on. Two vertex-only presets
   carry indices instead of values:
   1. `palette-index`: one index per vertex into a per-mesh table of the
      distinct flattened materials the mesh uses, written as `_PALETTEINDEX`.
   2. `palette-layers`: one index per layer per vertex, written as
      `_PALETTEINDEX0`, `_PALETTEINDEX1`, and so on, with each layer's
      palette shipped verbatim. A shader combines the indexed materials by
      the format's layer-override order: the last layer supplying a property
      wins. The only carrier that keeps every layer separately rather than
      the flattened per-property winners.
2. `--vertex-target <preset> <target>`: override a preset's attribute name,
   as `--vertex-target computed-occlusion _AO`. Repeatable.
3. `--vertex-map <target> <channels>`: a custom packing into `COLOR_0` or a
   `_NAME` attribute, sharing the [channel grammar](#channel-expressions) and
   `--define-property`.
4. `--palette-storage` `embedded` | `external` | `both`: where the palette
   tables go, glTF `extras.vxl` or a `<stem>-palette.json` sidecar carrying
   the identical JSON. Defaults like `--texture-storage`.

The palette data (examples in JSONC; a real file is plain JSON):

```jsonc
{
  "version": 1,
  "kind": "palette-index",
  "materials": [
    { "baseColor": "#FF0000FF", "roughness": 0.9 }, // index 0
    { "baseColor": "#FF0000FF", "roughness": 0.1 }, // index 1
    { "baseColor": "#0000FFFF", "roughness": 0.1 }  // index 2
  ]
}
```

```jsonc
{
  "version": 1,
  "kind": "palette-layers",
  "layers": [
    [ // layer 0 (base), indexed by _PALETTEINDEX0
      { "baseColor": "#FF0000FF" },
      { "baseColor": "#0000FFFF" }
    ],
    [ // layer 1 (finish), indexed by _PALETTEINDEX1
      { "roughness": 0.9 },
      { "roughness": 0.1 }
    ]
  ]
}
```

```typescript
type PaletteData = PaletteIndexData | PaletteLayersData;

interface PaletteIndexData {
  version: 1;
  kind: "palette-index";
  materials: Material[]; // indexed by _PALETTEINDEX
}

interface PaletteLayersData {
  version: 1;
  kind: "palette-layers";
  layers: Material[][]; // layers[n] indexed by _PALETTEINDEXn
}

// Voxel-json property keys to values: a color is a `#RRGGBBAA` hex string,
// a scalar a number. An omitted property takes its spec default.
type Material = { [property: string]: string | number };
```

### Computed occlusion and the unwrap atlas

`computed-occlusion` is occlusion computed from the voxel geometry, each face
corner reading the voxels that meet there. It varies across a surface, so as
a texture it needs `--atlas unwrap`, a per-mesh UV unwrap with a texel per
face. Under `--atlas palette` it would ride a second, unwrap UV set beside
the shared palette maps. As `--vertex computed-occlusion` it darkens
`COLOR_0`; greedy merging splits quads only where corner occlusion disagrees.
Three flags tune it wherever it is written:

1. `--computed-occlusion-strength <0..1>` (default `1.0`): scales how much it
   darkens.
2. `--computed-occlusion-min-brightness <0..1>` (default `0.0`): floor on the
   darkest crevice.
3. `--computed-occlusion-color-space` `linear` | `srgb` (default `linear`):
   the space the values are written in.

A sampled neighborhood model (a radius and a falloff curve) is a possible
extension beyond the discrete corner method.

### Scene assembly

A separate mode selecting hierarchy nodes and baking their transforms and
instancing into one placed mesh, complementing the pure-geometry object
selectors. With it comes a story for outputting more than one object.
