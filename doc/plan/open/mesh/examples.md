# Worked examples

_Part of the [mesh plan](README.md): runs, each a profile, the
command line that fires it, its expansion into flags, and the glTF it
produces._

Two small models carry every example. Every snippet spells its file
from the root down to the leaf it shows, the untouched keys folded
into one ellipsis comment, a large leaf collapsed to its own, and the
numbers are real: counts match the model and indices match the arrays
they point into. A `.glb`'s snippet is its JSON chunk, the binary
chunk riding behind it. A built-in profile lives in no file, so its
snippet roots at the map the binary embeds.

## The models

`lamp.voxj` is two voxels, a steel base with a glowing glass bulb on
top, the bulb above the base along glTF's `+Y`:

```jsonc
// lamp.voxj
{
  "version": 1,
  "main": {
    "runtimeState": {
      "valuePools": [
        {
          "kind": "vec-4-float",
          "values": [
            [0.5, 0.5, 0.5, 1],
            [1, 0.9, 0.6, 0.6],
          ],
        },
        { "kind": "float", "values": [0.9, 0.4] },
        { "kind": "float", "values": [1, 0] },
        {
          "kind": "vec-3-float",
          "values": [
            [0, 0, 0],
            [1, 0.9, 0.6],
          ],
        },
        { "kind": "float", "values": [0, 4] },
      ],
      "palettes": [
        {
          "properties": [
            { "name": "baseColorFactor", "valuePool": 0 },
            { "name": "roughnessFactor", "valuePool": 1 },
            { "name": "metallicFactor", "valuePool": 2 },
            { "name": "emissiveFactor", "valuePool": 3 },
            { "name": "emissiveStrength", "valuePool": 4 },
          ],
          "materials": [
            [0, 0, 0, 0, 0], // the steel base
            [1, 1, 1, 1, 1], // the glowing glass bulb
          ],
        },
      ],
      "rootNodes": [0],
      /* ... */
    },
  },
}
```

Greedy meshing merges only faces that share a material, so the stack
is ten faces: the bulb's top, the base's bottom, and each of the four
sides split at the seam. Ten quads are twenty triangles, and no face
shares corners with another, so the streams run forty vertices and
sixty indices.

`step.voxj` is three voxels of one stone material in an L, two on the
ground and one stacked on the left end, an inside corner where the
riser meets the tread:

```jsonc
// step.voxj
{
  "version": 1,
  "main": {
    "runtimeState": {
      "valuePools": [
        { "kind": "vec-4-float", "values": [[0.55, 0.5, 0.45, 1]] },
      ],
      "palettes": [
        {
          "properties": [{ "name": "baseColorFactor", "valuePool": 0 }],
          "materials": [
            [0], // the stone
          ],
        },
      ],
      "rootNodes": [0],
      /* ... */
    },
  },
}
```

Greedy meshing emits ten faces here too, by a different split: one
merged bottom, one merged left side, two quads each for the L-shaped
front and back, and two apiece for the tops and right sides the step
offsets from each other.

## Geometry alone

The bare run's profile is the empty one, every key omitted and every
default taken:

```jsonc
// .vxlconfig
{
  "mesh": {
    "profiles": {
      "geometry": {},
    },
  },
}
```

```sh
# the bare run
vxl mesh lamp.voxj

# the same file
vxl mesh lamp.voxj
  --profile geometry
```

Either writes `lamp.glb`, geometry only. No flag mentions a material,
so the file carries no `materials` array at all, and the implicit
primitive holds every face with no material, a viewer drawing it with
the spec's default. `NORMAL` writes by default beside
`POSITION`:

```jsonc
{
  "asset": { "version": "2.0" },
  "buffers": [{ "byteLength": 1080 }],
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 40,
      "type": "VEC3",
      "min": [0, 0, 0],
      "max": [1, 2, 1],
    }, // POSITION
    { "bufferView": 1, "componentType": 5126, "count": 40, "type": "VEC3" }, // NORMAL
    { "bufferView": 2, "componentType": 5123, "count": 60, "type": "SCALAR" }, // indices
  ],
  "meshes": [
    {
      "primitives": [
        { "attributes": { "POSITION": 0, "NORMAL": 1 }, "indices": 2 },
      ],
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

The buffer is the three streams packed: forty positions and forty
normals at twelve bytes each, sixty `u16` indices at two, 1080 bytes.
`--voxel-size` defaults to one meter, so positions run zero to
`[1, 2, 1]`.

## The pbr bake

The `pbr` profile ships
[built in](profile-language.md#built-in-profiles), so no `.vxlconfig`
is involved:

```jsonc
// the built-in profiles, the map the binary embeds
{
  "pbr": {
    "valuesFrom": ["albedo", "orm", "emissive"],
    "materials": [
      {
        "slots": {
          "baseColorTexture": { "kind": "value", "value": "albedo" },
          "occlusionTexture": { "kind": "value", "value": "orm" },
          "metallicRoughnessTexture": { "kind": "value", "value": "orm" },
          "emissiveTexture": { "kind": "value", "value": "emissive" },
          "emissiveFactor": { "kind": "value", "value": "white" },
          "emissiveStrength": { "kind": "value", "value": "maxStrength" },
        },
      },
    ],
  },
  /* ... */
}
```

```sh
vxl mesh lamp.voxj
  --profile pbr
```

or expanded into its flags, the material count deriving from the
mentions of material 0:

```sh
vxl mesh lamp.voxj
  --values-from albedo
  --values-from orm
  --values-from emissive
  --write-material-slot-value 0 baseColorTexture albedo
  --write-material-slot-value 0 occlusionTexture orm
  --write-material-slot-value 0 metallicRoughnessTexture orm
  --write-material-slot-value 0 emissiveTexture emissive
  --write-material-slot-value 0 emissiveFactor white
  --write-material-slot-value 0 emissiveStrength maxStrength
```

The profile evaluates its values over the two swatches: `albedo`
is the two base colors, `orm` packs occlusion 1 with roughness 0.9
and 0.4 and metallic 1 and 0, `emissive` is black for the steel and
`[1, 0.9, 0.6]` for the bulb, its strength 4 over the palette's
`maxStrength` of 4, and `white` pins `emissiveFactor` against glTF's
black default.

The [palette atlas](mesh.md#the-palette-atlas) lays one texel per
swatch. The default `pot` canvas is the smallest square power of two
holding two texels, 2 by 2, two cells used and two transparent black
the mesh never samples. Only swatch values write textures, so
the streams derive `[swatch]`, one `TEXCOORD_0`, each face's UVs at
its swatch's texel center: `(0.25, 0.25)` for the steel, `(0.75, 0.25)` for
the bulb.

```jsonc
{
  "asset": { "version": "2.0" },
  "extensionsUsed": ["KHR_materials_emissive_strength"],
  "accessors": [
    /* ... */
  ], // POSITION, NORMAL, TEXCOORD_0, indices
  "images": [
    { "mimeType": "image/png", "bufferView": 4 }, // albedo, sRGB
    { "mimeType": "image/png", "bufferView": 5 }, // orm, linear
    { "mimeType": "image/png", "bufferView": 6 }, // emissive, sRGB
  ],
  "samplers": [
    {
      "magFilter": 9728,
      "minFilter": 9728, // NEAREST
      "wrapS": 33071,
      "wrapT": 33071, // CLAMP_TO_EDGE
    },
  ],
  "textures": [
    { "sampler": 0, "source": 0 },
    { "sampler": 0, "source": 1 },
    { "sampler": 0, "source": 2 },
  ],
  "materials": [
    {
      "pbrMetallicRoughness": {
        "baseColorTexture": { "index": 0, "texCoord": 0 },
        "metallicRoughnessTexture": { "index": 1, "texCoord": 0 },
      },
      "occlusionTexture": { "index": 1, "texCoord": 0 },
      "emissiveTexture": { "index": 2, "texCoord": 0 },
      "emissiveFactor": [1, 1, 1],
      "extensions": {
        "KHR_materials_emissive_strength": { "emissiveStrength": 4 },
      },
    },
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
          "indices": 3,
          "material": 0,
        },
      ],
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

The orm image fills two slots from one embedded copy, and the
`emissiveStrength` slot lands as the extension with its use declared.
The bulb's 0.6 alpha rides the albedo texels, but `alphaMode` is
unwritten and defaults to `OPAQUE`, so the bulb draws solid; the
[computed enum](#a-computed-enum) below lifts it.

## A referenced file

A `.vxlconfig` beside the model defines a profile of its own, a
matte look that references its albedo map as a file instead of
embedding it, with the geometry riding the config too:

```jsonc
{
  "mesh": {
    "profiles": {
      "matte": {
        "valuesFrom": ["defaults"],
        "values": ["albedo = baseColorFactor"],
        "voxelSize": 0.1,
        "files": {
          "png": {
            "{file-stem}-albedo.png": { "transfer": "srgb", "value": "albedo" },
          },
        },
        "materials": [
          {
            "slots": {
              "baseColorTexture": {
                "kind": "file",
                "file": "{file-stem}-albedo.png",
              },
            },
          },
        ],
      },
    },
  },
}
```

```sh
vxl mesh lamp.voxj
  --to gltf
  --profile matte
```

or expanded into its flags, the file templates already filled:

```sh
vxl mesh lamp.voxj
  --to gltf
  --voxel-size 0.1
  --values-from matte
  --write-file-png-value lamp-albedo.png albedo srgb
  --write-material-slot-file 0 baseColorTexture lamp-albedo.png
```

The output resolves to `lamp.gltf`, so `{file-stem}` fills as `lamp`
and the run writes `lamp-albedo.png` beside the mesh. The writer's
`srgb` token cross-checks against `baseColorTexture`'s own sRGB
requirement, and the slot references the file by relative path:

```jsonc
{
  "asset": { "version": "2.0" },
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 40,
      "type": "VEC3",
      "min": [0, 0, 0],
      "max": [0.1, 0.2, 0.1],
    }, // POSITION, 0.1 meters per voxel
    /* ... */
  ],
  "images": [{ "uri": "lamp-albedo.png" }],
  "textures": [{ "sampler": 0, "source": 0 }],
  "materials": [
    {
      "pbrMetallicRoughness": {
        "baseColorTexture": { "index": 0, "texCoord": 0 },
      },
    },
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
          "indices": 3,
          "material": 0,
        },
      ],
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

`voxelSize` scales vertex positions alone, so the same forty vertices
now run to `[0.1, 0.2, 0.1]`.

## The palette pattern

The [palette pattern](mesh.md#palettes) serves a runtime that
resolves materials itself: rows under the mesh's extras, the index
they are read by on the primitive, no materials at all.

```jsonc
// .vxlconfig
{
  "mesh": {
    "profiles": {
      "palette": {
        "valuesFrom": ["albedo"],
        "primitives": [{ "indices": { "_PALETTE": "u8" } }],
        "meshExtras": {
          "albedo": {
            "kind": "json-value",
            "value": "albedo",
            "transfer": "linear",
          },
        },
      },
    },
  },
}
```

```sh
vxl mesh lamp.voxj
  --profile palette
```

or expanded into its flags, the primitives entry firing the explicit
no-material primitive:

```sh
vxl mesh lamp.voxj
  --values-from albedo
  --primitive none true
  --write-mesh-extra-json-value albedo albedo linear
  --write-primitive-index 0 _PALETTE u8
```

The built-in `albedo` profile reduces to one value here: a bare
`--value "albedo = baseColorFactor"` serves the same, the lamp's
palette supplying every base color.

```jsonc
{
  "asset": { "version": "2.0" },
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 40,
      "type": "VEC3",
      "min": [0, 0, 0],
      "max": [1, 2, 1],
    }, // POSITION
    { "bufferView": 1, "componentType": 5126, "count": 40, "type": "VEC3" }, // NORMAL
    { "bufferView": 2, "componentType": 5121, "count": 40, "type": "SCALAR" }, // _PALETTE
    { "bufferView": 3, "componentType": 5123, "count": 60, "type": "SCALAR" }, // indices
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": { "POSITION": 0, "NORMAL": 1, "_PALETTE": 2 },
          "indices": 3,
        },
      ],
      "extras": {
        "vxl": {
          "values": {
            "albedo": [
              [0.5, 0.5, 0.5, 1],
              [1, 0.9, 0.6, 0.6],
            ],
          },
        },
      },
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

The base's vertices hold index 0 and the bulb's hold 1, and a shader
reads `values.albedo[_PALETTE]` against rows the runtime can replace
at will. `NORMAL` stays: the runtime draws this
mesh itself, so it wants the lighting data, and dropping the stream
is the explicit `--write-primitive-normal 0 false`.

## Baked occlusion

Occlusion varies across a surface, not per swatch, so its
textures leave the palette layout: flat per face through the
[unwrap atlas](mesh.md#the-unwrap-atlas), or smooth per corner
through the [corner atlas](mesh.md#the-corner-atlas). This run bakes
the flat form, and `step.voxj` has the inside corner that makes it
visible:

```jsonc
// .vxlconfig: one profile binds the computation and bakes it
{
  "mesh": {
    "profiles": {
      "flat-ao": {
        "computeOcclusion": "computedOcclusion",
        "values": ["ao = faceAvg(computedOcclusion)"],
        "materials": [
          {
            "slots": {
              "occlusionTexture": { "kind": "value", "value": "ao" },
            },
          },
        ],
      },
    },
  },
}
```

```sh
vxl mesh step.voxj
  --profile flat-ao
```

or by hand:

```sh
vxl mesh step.voxj
  --compute-occlusion computedOcclusion
  --value "ao = faceAvg(computedOcclusion)"
  --write-material-slot-value 0 occlusionTexture ao
```

The corners along the riser's foot read below one, the open corners
read one, and `faceAvg` steps the corner value down to the ten
faces, one texel each, packed into the smallest power-of-two square
holding ten, 4 by 4. Only a face value writes a texture, so the
streams derive `[face]` and the one `TEXCOORD_0` is the face stream:

```jsonc
{
  "asset": { "version": "2.0" },
  "accessors": [
    /* ... */
  ], // POSITION, NORMAL, TEXCOORD_0, indices
  "images": [{ "mimeType": "image/png", "bufferView": 4 }],
  "textures": [{ "sampler": 0, "source": 0 }],
  "materials": [
    {
      "occlusionTexture": { "index": 0, "texCoord": 0 },
    },
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
          "indices": 3,
          "material": 0,
        },
      ],
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

The material carries exactly the one slot the run wrote, base color
falling to the spec's white default, so the mesh renders as lit stone
with darkened creases.

Written whole, the corner value skips the reduction and bakes the
[corner atlas](mesh.md#the-corner-atlas) instead:

```sh
vxl mesh step.voxj
  --compute-occlusion computedOcclusion
  --write-material-slot-value 0 occlusionTexture computedOcclusion
```

Ten faces are ten 2x2 blocks, forty texels in a 4-by-4 canvas of
cells, 8 by 8. The texture samples linear instead of nearest, the
streams derive `[corner]`, and each crease shades smooth across its
face instead of flat:

```jsonc
{
  "asset": { "version": "2.0" },
  "accessors": [
    /* ... */
  ], // POSITION, NORMAL, TEXCOORD_0, indices
  "images": [{ "mimeType": "image/png", "bufferView": 4 }], // 8x8, a block per face
  "samplers": [
    {
      "magFilter": 9729,
      "minFilter": 9729, // LINEAR, no mipmaps
      "wrapS": 33071,
      "wrapT": 33071, // CLAMP_TO_EDGE
    },
  ],
  "textures": [{ "sampler": 0, "source": 0 }],
  "materials": [
    {
      "occlusionTexture": { "index": 0, "texCoord": 0 },
    },
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
          "indices": 3,
          "material": 0,
        },
      ],
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

The `baked-ao` profile in the
[profile language](profile-language.md#user-defined-profiles) is this
run's config spelling, a `0.2` floor added.

## A lightmap stream

A stream can exist for texels the run never writes. An engine that
bakes its own lightmap wants per-face coordinates in the file, so
the primitive spells a face stream beside the swatch stream its
albedo
map reads, and the face entry stays textureless:

```jsonc
// .vxlconfig
{
  "mesh": {
    "profiles": {
      "lightmap": {
        "valuesFrom": ["albedo"],
        "materials": [
          {
            "slots": {
              "baseColorTexture": { "kind": "value", "value": "albedo" },
            },
          },
        ],
        "primitives": [{ "material": 0, "uvs": ["swatch", "face"] }],
      },
    },
  },
}
```

```sh
vxl mesh step.voxj
  --profile lightmap
```

or expanded into its flags:

```sh
vxl mesh step.voxj
  --values-from albedo
  --primitive 0 true
  --write-material-slot-value 0 baseColorTexture albedo
  --write-primitive-uv 0 swatch
  --write-primitive-uv 0 face
```

No `--uv` is spelled, so the stream list derives: the albedo texture
puts `swatch` in use, and the `face` mention joins it. The
primitive spells its streams, swatch then face, so `TEXCOORD_0` is
the swatch stream the material reads and `TEXCOORD_1` is the
[unwrap atlas](mesh.md#the-unwrap-atlas) layout, ten face cells in a
4-by-4 canvas with no image behind them. Unspelled, the primitive
would filter to the swatch stream alone and no face stream would
exist:

```jsonc
{
  "asset": { "version": "2.0" },
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 40,
      "type": "VEC3",
      "min": [0, 0, 0],
      "max": [2, 2, 1],
    }, // POSITION
    { "bufferView": 1, "componentType": 5126, "count": 40, "type": "VEC3" }, // NORMAL
    { "bufferView": 2, "componentType": 5126, "count": 40, "type": "VEC2" }, // TEXCOORD_0
    { "bufferView": 3, "componentType": 5126, "count": 40, "type": "VEC2" }, // TEXCOORD_1
    { "bufferView": 4, "componentType": 5123, "count": 60, "type": "SCALAR" }, // indices
  ],
  "images": [{ "mimeType": "image/png", "bufferView": 5 }], // albedo, 1x1
  "textures": [{ "sampler": 0, "source": 0 }],
  "materials": [
    {
      "pbrMetallicRoughness": {
        "baseColorTexture": { "index": 0, "texCoord": 0 },
      },
    },
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": {
            "POSITION": 0,
            "NORMAL": 1,
            "TEXCOORD_0": 2,
            "TEXCOORD_1": 3,
          },
          "indices": 4,
          "material": 0,
        },
      ],
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

The material's one texture names `texCoord: 0`, and nothing in the
file points at `TEXCOORD_1`: the engine bakes its lightmap into the
face layout and samples it by the stream the mesh already carries.

## One UV set

A consumer that reads one UV set wants one stream. `--uv` is the
bake contract, and listing `face` alone re-bakes every texture there.
The swatch albedo climbs, since the lowest listed domain at or above
swatch is face. The occlusion already lives there:

```jsonc
// .vxlconfig
{
  "mesh": {
    "profiles": {
      "one-uv": {
        "valuesFrom": ["albedo"],
        "computeOcclusion": "computedOcclusion",
        "values": ["ao = faceAvg(computedOcclusion)"],
        "uvs": ["face"],
        "materials": [
          {
            "slots": {
              "baseColorTexture": { "kind": "value", "value": "albedo" },
              "occlusionTexture": { "kind": "value", "value": "ao" },
            },
          },
        ],
      },
    },
  },
}
```

```sh
vxl mesh step.voxj
  --profile one-uv
```

or expanded into its flags:

```sh
vxl mesh step.voxj
  --values-from albedo
  --compute-occlusion computedOcclusion
  --value "ao = faceAvg(computedOcclusion)"
  --uv face
  --write-material-slot-value 0 baseColorTexture albedo
  --write-material-slot-value 0 occlusionTexture ao
```

With `--uv face`, both textures share the unwrap atlas layout, ten
cells in a 4 by 4. The albedo that filled a 1x1 palette canvas now
spends ten face texels saying the same stone color. That waste is the
price of the one stream:

```jsonc
{
  "asset": { "version": "2.0" },
  "accessors": [
    /* ... */
  ], // POSITION, NORMAL, TEXCOORD_0, indices
  "images": [
    { "mimeType": "image/png", "bufferView": 4 }, // albedo per face, sRGB
    { "mimeType": "image/png", "bufferView": 5 }, // ao, linear
  ],
  "textures": [
    { "sampler": 0, "source": 0 },
    { "sampler": 0, "source": 1 },
  ],
  "materials": [
    {
      "pbrMetallicRoughness": {
        "baseColorTexture": { "index": 0, "texCoord": 0 },
      },
      "occlusionTexture": { "index": 1, "texCoord": 0 },
    },
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
          "indices": 3,
          "material": 0,
        },
      ],
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

Both slots name `texCoord: 0`, the collapse the flag bought.

Without `--uv face`, the profile's `uvs` list, the streams derive
`[swatch, face]`, each texture at its value's own domain:

```jsonc
{
  "asset": { "version": "2.0" },
  "accessors": [
    /* ... */
  ], // POSITION, NORMAL, TEXCOORD_0, TEXCOORD_1, indices
  "images": [
    { "mimeType": "image/png", "bufferView": 5 }, // albedo, 1x1
    { "mimeType": "image/png", "bufferView": 6 }, // ao, 4x4
  ],
  "textures": [
    { "sampler": 0, "source": 0 },
    { "sampler": 0, "source": 1 },
  ],
  "materials": [
    {
      "pbrMetallicRoughness": {
        "baseColorTexture": { "index": 0, "texCoord": 0 }, // the swatch stream
      },
      "occlusionTexture": { "index": 1, "texCoord": 1 }, // the face stream
    },
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": {
            "POSITION": 0,
            "NORMAL": 1,
            "TEXCOORD_0": 2, // swatch
            "TEXCOORD_1": 3, // face
          },
          "indices": 4,
          "material": 0,
        },
      ],
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

The albedo returns to its 1x1 palette canvas, each slot samples its
own stream, and the consumer that reads one UV set is back to two.

## A crevice split

A runtime that draws weathering into the creases itself wants the
creased faces in their own primitive, bare of any material. Its
convention also pins face coordinates to `TEXCOORD_0`. The `--uv`
list orders the mesh's streams, and `--write-primitive-uv` gives the
bare primitive a stream its missing material would never pull in:

```jsonc
// .vxlconfig
{
  "mesh": {
    "profiles": {
      "crevice-split": {
        "valuesFrom": ["albedo"],
        "computeOcclusion": "computedOcclusion",
        "values": [
          "ao = faceAvg(computedOcclusion)",
          "crevice = ao < 0.9",
          "open = !crevice",
        ],
        "uvs": ["face", "swatch"],
        "materials": [
          {
            "slots": {
              "baseColorTexture": { "kind": "value", "value": "albedo" },
              "occlusionTexture": { "kind": "value", "value": "ao" },
            },
          },
        ],
        "primitives": [
          { "material": 0, "select": "open" },
          { "select": "crevice", "uvs": ["face"] },
        ],
      },
    },
  },
}
```

```sh
vxl mesh step.voxj
  --profile crevice-split
```

or expanded into its flags:

```sh
vxl mesh step.voxj
  --values-from albedo
  --compute-occlusion computedOcclusion
  --value "ao = faceAvg(computedOcclusion)"
  --value "crevice = ao < 0.9"
  --value "open = !crevice"
  --uv face
  --uv swatch
  --write-material-slot-value 0 baseColorTexture albedo
  --write-material-slot-value 0 occlusionTexture ao
  --primitive 0 open
  --primitive none crevice
  --write-primitive-uv 1 face
```

The tread and the riser meeting the inside corner average below one
and the other eight faces read one, so `crevice` takes exactly those
two faces and `open` the rest. The `[face, swatch]` list changes no
bake, since the lowest listed domain at or above swatch is still
swatch. It pins the order instead. Primitive 0 has no
`--write-primitive-uv`, so it filters the list to its material's
need, both domains. Face seats at `TEXCOORD_0`, swatch at
`TEXCOORD_1`, and the material's `texCoord`s derive to match.
Primitive 1 names its streams, so it writes exactly the face stream.
Its `none` material samples nothing, which makes the omitted swatch
legal. Both primitives' face UVs index the
one unwrap atlas, ten cells the mesh lays out once:

```jsonc
{
  "asset": { "version": "2.0" },
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 32,
      "type": "VEC3",
      "min": [0, 0, 0],
      "max": [2, 2, 1],
    }, // POSITION
    { "bufferView": 1, "componentType": 5126, "count": 32, "type": "VEC3" }, // NORMAL
    { "bufferView": 2, "componentType": 5126, "count": 32, "type": "VEC2" }, // face
    { "bufferView": 3, "componentType": 5126, "count": 32, "type": "VEC2" }, // swatch
    { "bufferView": 4, "componentType": 5123, "count": 48, "type": "SCALAR" }, // indices
    {
      "bufferView": 5,
      "componentType": 5126,
      "count": 8,
      "type": "VEC3",
      "min": [1, 1, 0],
      "max": [2, 2, 1],
    }, // POSITION
    { "bufferView": 6, "componentType": 5126, "count": 8, "type": "VEC3" }, // NORMAL
    { "bufferView": 7, "componentType": 5126, "count": 8, "type": "VEC2" }, // face
    { "bufferView": 8, "componentType": 5123, "count": 12, "type": "SCALAR" }, // indices
  ],
  "images": [
    { "mimeType": "image/png", "bufferView": 9 }, // albedo, 1x1
    { "mimeType": "image/png", "bufferView": 10 }, // ao, 4x4
  ],
  "textures": [
    { "sampler": 0, "source": 0 },
    { "sampler": 0, "source": 1 },
  ],
  "materials": [
    {
      "pbrMetallicRoughness": {
        "baseColorTexture": { "index": 0, "texCoord": 1 }, // the swatch stream
      },
      "occlusionTexture": { "index": 1, "texCoord": 0 }, // the face stream
    },
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": {
            "POSITION": 0,
            "NORMAL": 1,
            "TEXCOORD_0": 2, // face
            "TEXCOORD_1": 3, // swatch
          },
          "indices": 4,
          "material": 0,
        },
        {
          "attributes": {
            "POSITION": 5,
            "NORMAL": 6,
            "TEXCOORD_0": 7, // face, spelled for the runtime
          },
          "indices": 8,
        },
      ],
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

The two primitives partition the ten faces: thirty-two vertices draw
as stone, eight ride bare for the runtime's own pass, and every
face's `TEXCOORD_0` points into the same ten-cell layout.

Without the three UV flags, the profile's two `uvs` lists, the
defaults answer:

```jsonc
{
  "asset": { "version": "2.0" },
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 32,
      "type": "VEC3",
      "min": [0, 0, 0],
      "max": [2, 2, 1],
    }, // POSITION
    { "bufferView": 1, "componentType": 5126, "count": 32, "type": "VEC3" }, // NORMAL
    { "bufferView": 2, "componentType": 5126, "count": 32, "type": "VEC2" }, // swatch
    { "bufferView": 3, "componentType": 5126, "count": 32, "type": "VEC2" }, // face
    { "bufferView": 4, "componentType": 5123, "count": 48, "type": "SCALAR" }, // indices
    {
      "bufferView": 5,
      "componentType": 5126,
      "count": 8,
      "type": "VEC3",
      "min": [1, 1, 0],
      "max": [2, 2, 1],
    }, // POSITION
    { "bufferView": 6, "componentType": 5126, "count": 8, "type": "VEC3" }, // NORMAL
    { "bufferView": 7, "componentType": 5123, "count": 12, "type": "SCALAR" }, // indices
  ],
  "images": [
    { "mimeType": "image/png", "bufferView": 8 }, // albedo, 1x1
    { "mimeType": "image/png", "bufferView": 9 }, // ao, 4x4
  ],
  "textures": [
    { "sampler": 0, "source": 0 },
    { "sampler": 0, "source": 1 },
  ],
  "materials": [
    {
      "pbrMetallicRoughness": {
        "baseColorTexture": { "index": 0, "texCoord": 0 }, // the swatch stream
      },
      "occlusionTexture": { "index": 1, "texCoord": 1 }, // the face stream
    },
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": {
            "POSITION": 0,
            "NORMAL": 1,
            "TEXCOORD_0": 2, // swatch
            "TEXCOORD_1": 3, // face
          },
          "indices": 4,
          "material": 0,
        },
        {
          "attributes": { "POSITION": 5, "NORMAL": 6 }, // no UV stream
          "indices": 7,
        },
      ],
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

The derived list is `[swatch, face]`, ladder order, so face sits at
`TEXCOORD_1` and the runtime's `TEXCOORD_0` convention breaks. The
bare primitive's default filters to nothing, so the pass that wanted
face coordinates has none to read. The `--uv` pair pins the order,
and `--write-primitive-uv` writes the stream anyway.

## A computed enum

The lamp's bulb carries a 0.6 alpha the [pbr bake](#the-pbr-bake)
leaves opaque. An enum property takes a plain
[string](value-language.md#strings), so the mode is computed from the
palette itself:

```jsonc
// .vxlconfig; JSON escapes the inner quotes
{
  "mesh": {
    "profiles": {
      "glass": {
        "valuesFrom": ["albedo"],
        "values": [
          "mode = mix(\"OPAQUE\", \"BLEND\", min(baseColorFactor.a) < 1)",
        ],
        "materials": [
          {
            "slots": {
              "baseColorTexture": { "kind": "value", "value": "albedo" },
              "alphaMode": { "kind": "value", "value": "mode" },
            },
          },
        ],
      },
    },
  },
}
```

```sh
vxl mesh lamp.voxj
  --profile glass
```

or expanded into its flags, the `glass` values over the built-in
`albedo` profile they build on, the shell's single quotes carrying
the inner double quotes through:

```sh
vxl mesh lamp.voxj
  --values-from albedo
  --value 'mode = mix("OPAQUE", "BLEND", min(baseColorFactor.a) < 1)'
  --write-material-slot-value 0 baseColorTexture albedo
  --write-material-slot-value 0 alphaMode mode
```

`baseColorFactor.a` is the entries `[1, 0.6]`, `min` folds them to 0.6,
the comparison answers true, and `mix` picks `"BLEND"`. The writer
checks the token against glTF's own `alphaMode` list at the edge, so
a typo errors with the format named rather than landing in the file:

```jsonc
{
  "asset": { "version": "2.0" },
  "textures": [{ "sampler": 0, "source": 0 }],
  "materials": [
    {
      "pbrMetallicRoughness": {
        "baseColorTexture": { "index": 0, "texCoord": 0 },
      },
      "alphaMode": "BLEND",
    },
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0,
  /* ... */
}
```

The albedo texels carry the alpha, `BLEND` makes the viewer honor
it, and the bulb finally reads as glass.
