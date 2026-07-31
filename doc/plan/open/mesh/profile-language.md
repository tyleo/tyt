# Profile Language

Profiles for [`vxl mesh`](../vxl-commands/reference/mesh.md): named sets of values and outputs.
The [value language](value-language.md#profiles) documents the flags that
apply them: `--profile`, `--write-profile`, and `--stem`.

## Loading

User-defined profiles live in `.vxlconfig` files read through
tyt-preferences: one in the home directory and one at the git root. That
asks three things of the crate. Its `impl` feature currently pulls in
tyt-injection, which carries terminal, image, and network crates, and
loading a config needs only serde_json and an atomic file write, so
tyt-preferences should provide that impl itself; vxl then depends on
tyt-preferences alone, its first tyt crate. Its loaders currently
hardcode `.tytconfig`, so the file name becomes a parameter and every
tool names its own config file, vxl passing `.vxlconfig` and the `mesh`
key its envelope already spells. And its reads strip comments with
json_comments ahead of serde_json, so every config it loads is jsonc,
`.tytconfig` included. The stripper handles comments alone, so a
trailing comma stays the error strict JSON makes it.

Profiles resolve as a three-layer stack: the built-ins, then
`~/.vxlconfig`, then `<git-root>/.vxlconfig`. Each profile name is read
from the last layer that supplies it, wholesale, the rule the effective
palette already follows per property: a layer that respells a profile
respells all of it, so an override never inherits stray outputs from the
layer below. The layers merge into one namespace before `basedOn`
resolves, so a repo that overrides `defaults` changes every profile built
on it, including one from the home config.

Loading checks every profile in the merged namespace, so a broken config
fails the first run after the edit rather than the run that first names
the profile. Load-time checks are the ones that need no run context: the
schema's shape, every expression parsing, every `basedOn` name
resolving without a cycle, and no profile claiming one destination
twice. The rest wait for the run that decides them:
dimensions and shapes need the effective palette, slot names and their
encodings need the resolved output format, and symbol bindings need the
command line.

## Built-in profiles

Five profiles are built in: `defaults`, `albedo`, `orm`, `emissive`, and
the `pbr` bundle, the bottom layer of the [stack](#loading): `--profile
pbr` works before any `.vxlconfig` exists, and a config profile sharing a
built-in's name overrides it wholesale. The binary embeds this section's
map as data and parses it at startup with the config deserializer, so the
built-ins take the same schema by construction and every run exercises
the parse path. In the profile schema they are:

```jsonc
{
  // The glTF spec defaults, a mixin every profile builds on. Each entry
  // shadows its property with a defaulted copy.
  "defaults": {
    "values": [
      ["baseColorFactor", "default(baseColorFactor, rgba(1, 1, 1, 1))"],
      ["occlusionStrength", "default(occlusionStrength, 1)"],
      ["roughnessFactor", "default(roughnessFactor, 1)"],
      ["metallicFactor", "default(metallicFactor, 1)"],
      ["emissiveFactor", "default(emissiveFactor, rgb(0, 0, 0))"],
      ["emissiveStrength", "default(emissiveStrength, 1)"]
    ]
  },

  // outputs: one entry per value, each field named for the flag it
  // fires as and carrying that flag's arguments minus the value the
  // entry key names. png and json are { transfer, file } records, vertex
  // a { transfer, target } record for --write-vertex, slots the --slot
  // properties, slotExtras { transfer, name } records for --slot-extra;
  // slotFiles and slotExtraFiles reference the entry's
  // png instead of embedding. The reserved basedOn key opts into parent
  // outputs. The built-ins embed, so a slot fixes each encoding and no
  // entry carries a transfer.
  "albedo": {
    "basedOn": ["defaults"],
    "values": [["albedo", "baseColorFactor"]],
    "outputs": { "albedo": { "slots": ["baseColorTexture"] } }
  },

  // One value may fill several slots.
  "orm": {
    "basedOn": ["defaults"],
    "values": [
      ["orm", "rgb(occlusionStrength, roughnessFactor, metallicFactor)"]
    ],
    "outputs": {
      "orm": { "slots": ["occlusionTexture", "metallicRoughnessTexture"] }
    }
  },

  // white pins emissiveFactor against glTF's black default.
  "emissive": {
    "basedOn": ["defaults"],
    "values": [
      ["maxStrength", "max(emissiveStrength)"],
      [
        "emissive",
        "emissiveFactor * emissiveStrength / max(maxStrength, 0.001)"
      ],
      ["white", "rgb(1, 1, 1)"]
    ],
    "outputs": {
      "emissive": { "slots": ["emissiveTexture"] },
      "white": { "slots": ["emissiveFactor"] },
      "maxStrength": { "slots": ["emissiveStrength"] }
    }
  },

  // The bundle: the profile basedOn takes the members' values, the
  // outputs basedOn opts into their writes, and together they are the
  // whole profile.
  "pbr": {
    "basedOn": ["albedo", "orm", "emissive"],
    "outputs": { "basedOn": ["albedo", "orm", "emissive"] }
  }
}
```

## User-defined profiles

Every other profile lives under `.vxlconfig`'s `mesh.profiles` key, in the
same schema, and may build on the built-ins. `mse` packs metallic,
smoothness, and normalized emissive strength into one mask; `orm-files`
flips a built-in's slots from embedding to referencing; `heat` feeds a
runtime of your own through the custom slot; `vertex-colors` skips
textures and rides base color on the vertices; `baked-ao` bakes
[computed occlusion](value-language.md#computed-occlusion) into the
standard slot:

```jsonc
{
  "mesh": {
    "profiles": {
      // emissiveStrength is unbounded, so the mask normalizes by the
      // palette's strongest strength and the raw intensity rides the
      // material slot.
      "mse": {
        "basedOn": ["defaults"],
        "values": [
          ["smoothness", "1 - roughnessFactor"],
          ["maxStrength", "max(emissiveStrength)"],
          [
            "mse",
            "rgb(metallicFactor, smoothness, emissiveStrength / max(maxStrength, 0.001))"
          ]
        ],
        "outputs": {
          "mse": {
            "png": { "transfer": "linear", "file": "{stem}-mse.png" }
          },
          "maxStrength": { "slots": ["emissiveStrength"] }
        }
      },

      // slotFiles references the written png where slots would embed.
      "orm-files": {
        "basedOn": ["orm"],
        "outputs": {
          "orm": {
            "png": { "transfer": "linear", "file": "{stem}-orm.png" },
            "slotFiles": ["occlusionTexture", "metallicRoughnessTexture"]
          }
        }
      },

      // A per-material heat mask and one accent color, entries a runtime
      // of your own looks up under extras.vxl. The heat png writes and
      // its extra references it; the plain accent inlines its numbers.
      "heat": {
        "basedOn": ["defaults"],
        "values": [
          ["heat", "step(0.001, emissiveStrength)"],
          ["accent", "avg(baseColorFactor.rgb)"]
        ],
        "outputs": {
          "heat": {
            "png": { "transfer": "linear", "file": "{stem}-heat.png" },
            "slotExtraFiles": ["heat"]
          },
          "accent": {
            "slotExtras": [{ "transfer": "srgb", "name": "accent" }]
          }
        }
      },

      // No textures: base color rides the vertices as COLOR_0. Deferred
      // with the vertex writer.
      "vertex-colors": {
        "basedOn": ["defaults"],
        "values": [["albedo", "baseColorFactor"]],
        "outputs": {
          "albedo": {
            "vertex": { "transfer": "linear", "target": "COLOR_0" }
          }
        }
      },

      // Computed occlusion into the standard slot, floored at 0.2. It
      // reads no palette property, so no defaults mixin. Deferred with
      // the unwrap atlas and the per-face shape.
      "baked-ao": {
        "values": [["ao", "max(computedOcclusion, 0.2)"]],
        "outputs": {
          "ao": { "slots": ["occlusionTexture"] }
        }
      }
    }
  }
}
```

## Open questions

The open questions for this page and the
[value language](value-language.md) live on
[their own page](open-questions.md).
