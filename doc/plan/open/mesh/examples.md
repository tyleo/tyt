# Worked examples

_Part of the [mesh plan](README.md): six runs, each a command line or
config and the glTF it produces._

Two small models carry every example. A glTF snippet spells the file
from the root down to the leaf it shows, a large leaf collapsed to an
ellipsis comment, and the numbers are real: counts match the model and
indices match the arrays they point into. A `.glb`'s snippet is its
JSON chunk, the binary chunk riding behind it.

## The models

`lamp.voxj` is two voxels, a steel base with a glowing glass bulb on
top, the bulb above the base along glTF's `+Y`:

```jsonc
// lamp.voxj's palette
{ "baseColorFactor": [0.5, 0.5, 0.5, 1], "roughnessFactor": 0.9,
  "metallicFactor": 1, "emissiveStrength": 0 },
{ "baseColorFactor": [1, 0.9, 0.6, 0.6], "roughnessFactor": 0.4,
  "metallicFactor": 0, "emissiveFactor": [1, 0.9, 0.6],
  "emissiveStrength": 4 }
```

Greedy meshing merges only faces that share a material, so the stack
is ten faces: the bulb's top, the base's bottom, and each of the four
sides split at the seam. Ten quads are twenty triangles, and no face
shares corners with another, so the streams run forty vertices and
sixty indices.

`step.voxj` is three voxels of one stone material in an L, two on the
ground and one stacked on the left end, an inside corner where the
riser meets the tread. Greedy meshing emits ten faces here too, by a
different split: one merged bottom, one merged left side, two quads
each for the L-shaped front and back, and two apiece for the tops and
right sides the step offsets from each other.

## Geometry alone

```
vxl mesh lamp.voxj
```

writes `lamp.glb`, geometry only. No `--material-count` means no
materials, so the file carries no `materials` array at all, and the
implicit primitive holds every face with no material, a viewer drawing
it with the spec's default. `NORMAL` writes by default beside
`POSITION`:

```jsonc
{
  "asset": { "version": "2.0" },
  "buffers": [ { "byteLength": 1080 } ],
  "bufferViews": [ /* ... */ ],
  "accessors": [
    { "bufferView": 0, "componentType": 5126, "count": 40, "type": "VEC3",
      "min": [0, 0, 0], "max": [1, 2, 1] },                                  // POSITION
    { "bufferView": 1, "componentType": 5126, "count": 40, "type": "VEC3" }, // NORMAL
    { "bufferView": 2, "componentType": 5123, "count": 60, "type": "SCALAR" } // indices
  ],
  "meshes": [ { "primitives": [
    { "attributes": { "POSITION": 0, "NORMAL": 1 }, "indices": 2 }
  ] } ],
  "nodes": [ { "mesh": 0 } ],
  "scenes": [ { "nodes": [ 0 ] } ],
  "scene": 0
}
```

The buffer is the three streams packed: forty positions and forty
normals at twelve bytes each, sixty `u16` indices at two, 1080 bytes.
`--voxel-size` defaults to one meter, so positions run zero to
`[1, 2, 1]`.

## The pbr bake

```
vxl mesh lamp.voxj --output-profile pbr
```

The built-in [`pbr` profile](profile-language.md#built-in-profiles)
evaluates its values over the two palette rows: `albedo` is the two
base colors, `orm` packs occlusion 1 with roughness 0.9 and 0.4 and
metallic 1 and 0, `emissive` is black for the steel and `[1, 0.9,
0.6]` for the bulb, its strength 4 over the palette's `maxStrength`
of 4, and `white` pins `emissiveFactor` against glTF's black default.

The [palette atlas](mesh.md#the-palette-atlas) lays one texel per
row. The default `pot` canvas is the smallest square power of two
holding two texels, 2 by 2, two cells used and two transparent black
the mesh never samples. Only row values write textures, so the
streams derive `[row]`, one `TEXCOORD_0`, each face's UVs at its
row's texel center: `(0.25, 0.25)` for the steel, `(0.75, 0.25)` for
the bulb.

```jsonc
{
  "asset": { "version": "2.0" },
  "extensionsUsed": [ "KHR_materials_emissive_strength" ],
  "buffers": [ /* ... */ ],
  "bufferViews": [ /* ... */ ],
  "accessors": [ /* ... */ ],   // POSITION, NORMAL, TEXCOORD_0, indices
  "images": [
    { "mimeType": "image/png", "bufferView": 4 },   // albedo, sRGB
    { "mimeType": "image/png", "bufferView": 5 },   // orm, linear
    { "mimeType": "image/png", "bufferView": 6 }    // emissive, sRGB
  ],
  "samplers": [ {
    "magFilter": 9728, "minFilter": 9728,   // NEAREST
    "wrapS": 33071, "wrapT": 33071          // CLAMP_TO_EDGE
  } ],
  "textures": [
    { "sampler": 0, "source": 0 },
    { "sampler": 0, "source": 1 },
    { "sampler": 0, "source": 2 }
  ],
  "materials": [ {
    "pbrMetallicRoughness": {
      "baseColorTexture": { "index": 0, "texCoord": 0 },
      "metallicRoughnessTexture": { "index": 1, "texCoord": 0 }
    },
    "occlusionTexture": { "index": 1, "texCoord": 0 },
    "emissiveTexture": { "index": 2, "texCoord": 0 },
    "emissiveFactor": [ 1, 1, 1 ],
    "extensions": {
      "KHR_materials_emissive_strength": { "emissiveStrength": 4 }
    }
  } ],
  "meshes": [ { "primitives": [ {
    "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
    "indices": 3,
    "material": 0
  } ] } ],
  "nodes": [ { "mesh": 0 } ],
  "scenes": [ { "nodes": [ 0 ] } ],
  "scene": 0
}
```

The orm image fills two slots from one embedded copy, and the
`emissiveStrength` slot lands as the extension with its use declared.
The bulb's 0.6 alpha rides the albedo texels, but `alphaMode` is
unwritten and defaults to `OPAQUE`, so the bulb draws solid; the
[computed enum](#a-computed-enum) below lifts it.

## A run from config

A `.vxlconfig` beside the model defines a profile pair of its own,
a matte look that references its albedo map as a file instead of
embedding it, with the geometry riding the config too:

```jsonc
{
  "mesh": {
    "valueProfiles": {
      "matte": {
        "basedOn": ["defaults"],
        "values": [["albedo", "baseColorFactor"]]
      }
    },
    "outputProfiles": {
      "matte": {
        "voxelSize": 0.1,
        "values": ["matte"],
        "files": {
          "png": {
            "{file-stem}-albedo.png": { "transfer": "srgb", "value": "albedo" }
          }
        },
        "materials": [ {
          "slots": {
            "baseColorTexture": { "kind": "file", "file": "{file-stem}-albedo.png" }
          }
        } ]
      }
    }
  }
}
```

```
vxl mesh lamp.voxj --to gltf --output-profile matte
```

The output resolves to `lamp.gltf`, so `{file-stem}` fills as `lamp`
and the run writes `lamp-albedo.png` beside the mesh. The writer's
`srgb` token cross-checks against `baseColorTexture`'s own sRGB
requirement, and the slot references the file by relative path:

```jsonc
{
  "asset": { "version": "2.0" },
  "buffers": [ /* ... */ ],
  "bufferViews": [ /* ... */ ],
  "accessors": [
    { "bufferView": 0, "componentType": 5126, "count": 40, "type": "VEC3",
      "min": [0, 0, 0], "max": [0.1, 0.2, 0.1] },   // POSITION, 0.1 meters per voxel
    /* ... */
  ],
  "images": [ { "uri": "lamp-albedo.png" } ],
  "samplers": [ /* ... */ ],
  "textures": [ { "sampler": 0, "source": 0 } ],
  "materials": [ {
    "pbrMetallicRoughness": {
      "baseColorTexture": { "index": 0, "texCoord": 0 }
    }
  } ],
  "meshes": [ { "primitives": [ {
    "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
    "indices": 3,
    "material": 0
  } ] } ],
  "nodes": [ { "mesh": 0 } ],
  "scenes": [ { "nodes": [ 0 ] } ],
  "scene": 0
}
```

`voxelSize` scales vertex positions alone, so the same forty vertices
now run to `[0.1, 0.2, 0.1]`.

## The palette pattern

The [palette pattern](mesh.md#palettes) serves a runtime that
resolves materials itself: rows under the mesh's extras, the index
they are read by on the primitive, no materials at all.

```
vxl mesh lamp.voxj
    --value albedo "baseColorFactor"
    --write-mesh-extra-json-value albedo albedo linear
    --write-primitive-index 0 _PALETTE u8
```

```jsonc
{
  "asset": { "version": "2.0" },
  "buffers": [ /* ... */ ],
  "bufferViews": [ /* ... */ ],
  "accessors": [
    { "bufferView": 0, "componentType": 5126, "count": 40, "type": "VEC3",
      "min": [0, 0, 0], "max": [1, 2, 1] },                                  // POSITION
    { "bufferView": 1, "componentType": 5126, "count": 40, "type": "VEC3" }, // NORMAL
    { "bufferView": 2, "componentType": 5121, "count": 40, "type": "SCALAR" }, // _PALETTE
    { "bufferView": 3, "componentType": 5123, "count": 60, "type": "SCALAR" } // indices
  ],
  "meshes": [ {
    "primitives": [ {
      "attributes": { "POSITION": 0, "NORMAL": 1, "_PALETTE": 2 },
      "indices": 3
    } ],
    "extras": { "vxl": { "values": {
      "albedo": [ [0.5, 0.5, 0.5, 1], [1, 0.9, 0.6, 0.6] ]
    } } }
  } ],
  "nodes": [ { "mesh": 0 } ],
  "scenes": [ { "nodes": [ 0 ] } ],
  "scene": 0
}
```

The base's vertices hold index 0 and the bulb's hold 1, and a shader
reads `values.albedo[_PALETTE]` against rows the runtime can replace
at will. `NORMAL` stays: the runtime draws this
mesh itself, so it wants the lighting data, and dropping the stream
is the explicit `--write-primitive-normal 0 false`.

## The unwrap atlas

Occlusion varies across a surface, not per palette row, so it wants
the [unwrap atlas](mesh.md#the-unwrap-atlas), a texel per face, and
`step.voxj` has the inside corner that makes it visible:

```
vxl mesh step.voxj --atlas unwrap
    --compute-occlusion computedOcclusion
    --value ao "faceAverage(computedOcclusion)"
    --material-count 1
    --write-material-slot-value 0 occlusionTexture ao
```

The corners along the riser's foot read below one, the open corners
read one, and `faceAverage` steps the corner value down to the ten
faces, one texel each, packed into the smallest power-of-two square
holding ten, 4 by 4. Only a face value writes a texture, so the
streams derive `[face]` and the one `TEXCOORD_0` is the face stream:

```jsonc
{
  "asset": { "version": "2.0" },
  "buffers": [ /* ... */ ],
  "bufferViews": [ /* ... */ ],
  "accessors": [ /* ... */ ],   // POSITION, NORMAL, TEXCOORD_0, indices
  "images": [ { "mimeType": "image/png", "bufferView": 4 } ],
  "samplers": [ /* ... */ ],
  "textures": [ { "sampler": 0, "source": 0 } ],
  "materials": [ {
    "occlusionTexture": { "index": 0, "texCoord": 0 }
  } ],
  "meshes": [ { "primitives": [ {
    "attributes": { "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 },
    "indices": 3,
    "material": 0
  } ] } ],
  "nodes": [ { "mesh": 0 } ],
  "scenes": [ { "nodes": [ 0 ] } ],
  "scene": 0
}
```

The material carries exactly the one slot the run wrote, base color
falling to the spec's white default, so the mesh renders as lit stone
with darkened creases.

## A computed enum

The lamp's bulb carries a 0.6 alpha the [pbr bake](#the-pbr-bake)
leaves opaque. An enum property takes a plain
[string](value-language.md#strings), so the mode is computed from the
palette itself:

```
vxl mesh lamp.voxj --output-profile albedo
    --value mode 'mix("OPAQUE", "BLEND", min(baseColorFactor.a) < 1)'
    --write-material-slot-value 0 alphaMode mode
```

`baseColorFactor.a` is the rows `[1, 0.6]`, `min` folds them to 0.6,
the comparison answers true, and `mix` picks `"BLEND"`. The writer
checks the token against glTF's own `alphaMode` list at the edge, so
a typo errors with the format named rather than landing in the file:

```jsonc
{
  "asset": { "version": "2.0" },
  "buffers": [ /* ... */ ],
  "bufferViews": [ /* ... */ ],
  "accessors": [ /* ... */ ],
  "images": [ /* ... */ ],
  "samplers": [ /* ... */ ],
  "textures": [ { "sampler": 0, "source": 0 } ],
  "materials": [ {
    "pbrMetallicRoughness": {
      "baseColorTexture": { "index": 0, "texCoord": 0 }
    },
    "alphaMode": "BLEND"
  } ],
  "meshes": [ /* ... */ ],
  "nodes": [ { "mesh": 0 } ],
  "scenes": [ { "nodes": [ 0 ] } ],
  "scene": 0
}
```

The albedo texels carry the alpha, `BLEND` makes the viewer honor
it, and the bulb finally reads as glass.
