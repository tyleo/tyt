# Palette material tools

Status: **open**, drafted 2026-10-18 when
[vmax strictness](../../closed/vmax-strictness/README.md) closed. Nothing is
designed yet. This page collects the questions to settle before a design.

## Rule

A `.voxj` edit can leave a document that a target format cannot hold. A
palette might need 9 Voxel Max material slots, for example. The writer never
reshapes the document to fit. It errors, saying what does not fit and where. A
`vxl` command then repairs the document.

## Voxel Max Slots

A Voxel Max palette holds 256 colors and 8 material slots. A slot holds
metallic, roughness, emission strength, transmission, IOR, absorption, and
shadows. Each voxel stores one color and one slot. Many colors can share a
slot.

voxcore has no slots. The loader stores a slot as one value id that all of a
material's material-axis properties share, and the writer reads that id back.
Since `803f5d16`, `vxl palette edit` keeps the sharing intact. A slot breaks
apart only when its materials write different results.

## Gaps

| The vmax writer refuses | Repair command |
|---|---|
| More than 256 colors | `vxl palette quantize --max-materials 255` |
| A glowing material whose emissive color differs from its base color | `vxl palette edit --write-property emissiveColor 'baseColor.rgb'` |
| More than 8 material slots | None |
| A material that an uneven edit pulled out of its slot | None |

Removing a material has no command either. `palette quantize` drops only the
materials no voxel samples.

## Questions

1. **A material command group.** The value language picks materials by
   content, so `vxl palette edit` cannot target material 3. A
   `vxl palette material` group, following the noun-first commands, could hold
   `list`, `edit`, and `remove`. `list` would show each material's color and
   the values it shares with other materials. In a Voxel Max palette, those
   shared values are its slot
2. **What a slot edit addresses.** An edit can address the materials in a
   slot, such as every material in slot 2. It can instead address a value,
   such as roughness value 2, which changes every material drawing it. The
   value form works in every format. In a Voxel Max palette, it edits exactly
   one slot
3. **What a removed material's voxels become.** Removing a material leaves its
   voxels drawing nothing. The command could clear them, or move them onto
   another material it takes as an argument
4. **Repairing slots.** No command fixes a palette needing more than 8 slots,
   or a material an uneven edit pulled out of its slot. Moving a material into
   a free slot is the obvious repair. It needs a rule for picking the slot. It
   also needs a rule for what the new slot takes from the ext: the
   transmission color and whether its dispersion was stored
