# glTF ranges for the voxj property vocabulary

The exact schema wording behind the ranges the
[README](../README.md#where-ranges-live) moves out of the format and into
voxsmith. Read from the Khronos glTF `main` schemas on 2026-08-01, so
iteration 1's spec Range column and iteration 7's vocabulary check spell
the same thing without either one repeating the lookup.

| voxj property       | glTF schema                                        | Range              | Default        |
| ------------------- | -------------------------------------------------- | ------------------ | -------------- |
| `baseColor`         | `material.pbrMetallicRoughness` `baseColorFactor`   | 4 numbers, `[0,1]` | `[1,1,1,1]`    |
| `metallic`          | `material.pbrMetallicRoughness` `metallicFactor`    | `[0,1]`            | `1`            |
| `roughness`         | `material.pbrMetallicRoughness` `roughnessFactor`   | `[0,1]`            | `1`            |
| `emissiveColor`     | `material` `emissiveFactor`                         | 3 numbers, `[0,1]` | `[0,0,0]`      |
| `occlusionStrength` | `material.occlusionTextureInfo` `strength`          | `[0,1]`            | `1`            |
| `emissiveStrength`  | `KHR_materials_emissive_strength`                   | `[0, inf)`         | `1`            |
| `ior`               | `KHR_materials_ior`                                 | `{0}` or `[1, inf)`| `1.5`          |
| `transmission`      | `KHR_materials_transmission` `transmissionFactor`   | `[0,1]`            | `0`            |

Every range above is a schema `minimum` and `maximum` on a `number`, and
the two color rows put theirs on the array's `items` alongside `minItems`
and `maxItems` of 4 and 3.

`ior` is the one that is not an interval, and the schema says so
structurally rather than in prose: the property is a `oneOf` over exactly
`0.0` (`minimum` and `maximum` both `0.0`) and `minimum: 1.0` with no
upper bound. So the README's `{0} union [1, inf)` is the schema's own
shape, not a reading of it. `emissiveStrength` is the other open end,
with a `minimum` and no `maximum`.

Two places disagree with this table today and iteration 7 reconciles
them: `voxsmith/src/gltf_attributes.rs`'s `scalar_range` gives `ior`
`(1.0, None)`, which rejects `0`, and `internal/mesh/mesh_material.rs`'s
`ior` doc says `1+`.
