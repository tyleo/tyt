# Conventions and cross-command options

*Part of the [Vxl Command-Line Reference](../README.md).*

These hold across the commands and match the existing `vox-doc to` commands.

1. Input format is recognized by leading bytes or inferred from the extension,
   and overridden with `--from`. Mesh I/O format is inferred from the mesh
   extension or set with `--to` and `--from`.
2. Output paths are optional and default to the input stem with the new
   extension, so a defaulted `vox-doc to voxj` writes `.voxj` and `--format zip`
   writes `.voxjz`.
3. Settable booleans follow the `--ext` style: a bare flag means `true`, an
   explicit `--flag false` turns it off, and the option has a default.
4. Palette addressing differs by command. [`palette list`](palette/list.md)
   takes repeatable positional index filters such as `1` or `1-5`.
   [`palette show`](palette/show.md) takes a repeatable
   `--property <palette> <property> <presentation> <reading>` selector,
   defaulting to the whole-document wildcard `'*' '*' auto auto`, and extends
   `--property` with the `<key>.component` grammar to read one component of a
   vector through either alias set (`.r`/`.g`/`.b`/`.a` or
   `.x`/`.y`/`.z`/`.w`). The mutating `quantize` and `remap` address a
   palette with `--index` (default `0`) and `--property` (default
   `baseColor`), operate on a whole property, and reject a component.
   Property keys are the glTF vocabulary names such as `baseColor`.
5. The read-only reports render with `--layout`, every command drawing its
   values from one shared vocabulary whose prefix names the output family:
   `box-hierarchy` and `box-tables` draw with box glyphs, `json-compact` and
   `json-pretty` serialize, `md-lists` and `md-tables` emit markdown, and
   `text-columns` and `text-rows` pad plain text.
   [`palette show`](palette/show.md) offers all eight, defaults to `text-rows`,
   and refines them with `--label`, `--header-level`, and `--table-shape`.
   [`node list`](node/list.md) offers `box-hierarchy` (its default),
   `json-compact`, and `json-pretty`. `vox-doc validate` offers `json-compact`,
   `json-pretty`, and `md-tables` (its default); `vox-doc show` adds
   `box-tables` to those three, and [`palette list`](palette/list.md) adds
   `box-hierarchy` (its default) and `box-tables`. `profile object mesh list`,
   `profile object voxels quantize list`, `profile palette quantize list`, and
   `profile palette show list` offer `box-hierarchy` (their default),
   `box-tables`, `json-compact`, `json-pretty`, `md-lists`, `md-tables`, or
   `text-rows`.
6. Every profile takes an optional one-line `description`. The profile `list`
   commands print it beside the profile's name unless
   `--show-descriptions false` turns it off.
7. Multiple values are passed by repeating the flag, as in
   `--select-index 0 --select-index 3`, not as one comma-separated argument. The
   exception is the `--texture-map` channel list, where the comma-separated RGBA
   packing is a single structured value.

## Glob patterns

Patterns follow `.gitignore` rules, not grep substring matching. The
[`node list`](node/list.md) patterns and the `--select` path glob share one rule
set, matched by the `pathspec` engine:

1. A pattern is a full match against a whole path segment, not a substring. `door`
   matches the name `door`, not `backdoor`; write `*door*` for a substring match.
2. `*` and `?` match within a segment and never cross `/`; `**` crosses `/`.
   `[...]` is a character class.
3. A pattern with no `/` floats and matches at any depth; a leading or interior
   `/` anchors it to the root, and `**` also reaches any depth.
4. A leading `!` negates a pattern, so a later pattern subtracts from an earlier
   one. A trailing `/` matches nodes (directories) only, never a leaf object by
   its own name.
5. Commands take several patterns, and the last one to match a path decides
   whether it is selected. Selecting a node selects its whole subtree; an
   excluded node prunes its subtree, so a subtracted branch cannot be re-added
   piecemeal.

## Object selectors

[`object mesh`](../../../../ref/mesh/mesh.md) and
[`object material`](object/material.md) choose which objects to output,
[`object render`](../../../../ref/render/render.md) which objects to draw,
[`vox-doc to`](vox-doc/to/README.md) which objects to write,
[`vox-doc show`](vox-doc/show.md) which objects to report,
[`object voxels quantize`](object/voxels/quantize.md) which objects to
quantize, and [`palette remap`](palette/remap.md) which objects to dither, with
two repeatable options, one per addressing mode, so a value is never parsed as
either an index or a glob. Selection targets objects; under `object mesh` and
`object render` each matched object lands placed by the hierarchy nodes
reaching it, so a path is the selection key and the placement follows from the
document.

1. `--select-index <index>`: an object index into the document's `objects`,
   a plain integer such as `0` or a range `a-b` such as `2-5`. Repeat the flag
   to pick several, as in `--select-index 0 --select-index 3`. Index is the
   canonical object reference in the spec.
2. `--select <glob>`: a glob over hierarchy paths, matched with the shared
   [glob rules](#glob-patterns) exactly as [`node list`](node/list.md) matches
   node paths. The candidates are the path of every node and every object it
   places: a node's path is the chain of node names from a root, an object's
   path that chain plus the object. A match selects every object at or under it,
   so matching a node selects its whole subtree, just as selecting a node in
   `node list` brings in its subtree, and matching an object selects that
   object. `--select a` selects every object under node `a`, `--select a/**` the
   same by its descendants, and `--select a/b` only object `b`. The graph is a
   DAG, so an object reached through several parents has one path per placement
   and matches when any path does; an object no node references has just its
   name as its path. Names are not unique, so a glob may match several objects.

Both options repeat, and every `--select-index` and `--select` value unions its
matches. Given neither, every object is selected; given one that matches
nothing, the command errors rather than quietly selecting nothing.
`object material` outputs the selection, `vox-doc to` writes it, `vox-doc show`
reports it, `object voxels quantize` quantizes it, `remap` dithers it,
`object render` draws it in every view (see
[render](../../../../ref/render/render.md)), and `object mesh` outputs it into
one mesh, or one per object under `--split-files` (see
[mesh](../../../../ref/mesh/mesh.md)).

Baking a matched node's subtree and transforms into one flattened mesh, rather
than carrying the nodes over, is a separate mode left for a later pass.
