# `vxl palette`

*Part of the [Vxl Command-Line Reference](../../README.md).*

Palette operations. Addressing is per command: [`list`](list.md) selects
palettes by positional index filters such as `1` or `1-5`, and [`show`](show.md)
selects with a repeatable `--property <palette> <property> <format>` selector
that defaults to the whole-document wildcard `'*' '*' auto`. The mutating
[`quantize`](quantize.md) and [`remap`](remap.md) address a palette with
`--index` (default `0`) and `--property` (default `baseColorFactor`). Property
keys are the glTF names such as `baseColorFactor`, not the old `rgba`.

- [`vxl palette list`](list.md): overview of every palette in a document.
- [`vxl palette show`](show.md): print one palette's selected properties.
- [`vxl palette quantize`](quantize.md): reduce a palette's colors.
- [`vxl palette remap`](remap.md): remap voxels onto a target palette.
