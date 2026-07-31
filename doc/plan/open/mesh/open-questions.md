# Open questions

_A subplan of [`vxl mesh`](../vxl-commands/reference/mesh.md): the open questions left by the
[value language](value-language.md) and the
[profile language](profile-language.md)._

## The wider plan

1. The ordered steps. The language pages are the design: there is no
   checklist and no schedule for building
   [the language crate](value-language.md#the-language-crate), for
   deleting voxsmith's `MaterialSlot::OcclusionMetallicRoughness`, which two
   slots naming one value replace, for growing `png_bytes.rs` past its
   hardcoded RGBA to the three other color types the sized-to-value rule
   needs, for the crate work [Loading](profile-language.md#loading) asks of
   tyt-preferences, or for deleting the `extras.vxl.maps` emission, which
   no flag can produce anymore.
2. [`vxl material`](../vxl-commands/reference/material.md), whose reference still takes the retired
   map flags and still says a selection may cover several objects, which a
   per-object atlas cannot serve. Deliberately deferred.
