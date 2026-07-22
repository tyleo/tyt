# Conventions and cross-command options

*Part of the [Vxl Command-Line Reference](../README.md).*

These hold across the commands and match the existing `to` commands.

1. Input format is recognized by leading bytes or inferred from the extension,
   and overridden with `--from`. Mesh I/O format is inferred from the mesh
   extension or set with `--to` and `--from`.
2. Output paths are optional and default to the input stem with the new
   extension, so a defaulted `to voxj` writes `.voxj` and `--format zip` writes
   `.voxjz`.
3. Settable booleans follow the `--ext` style: a bare flag means `true`, an
   explicit `--flag false` turns it off, and the option has a default.
4. Palette addressing differs by command. [`palette list`](palette/list.md)
   takes repeatable positional index filters such as `1` or `1-5`.
   [`palette show`](palette/show.md) takes a repeatable
   `--property <palette> <property> <format>` selector, defaulting to the
   whole-document wildcard `'*' '*' auto`, and extends `--property` with the
   `<key>.component` grammar to read one channel of a color. The mutating
   `quantize` and `remap` address a palette with `--index` (default `0`) and
   `--property` (default `baseColorFactor`), operate on a whole property, and
   reject a component. Property keys are the glTF names such as `baseColorFactor`,
   not the old `rgba`.
5. The read-only reports render with `--layout`.
   [`palette show`](palette/show.md) uses the shared layout vocabulary,
   `hierarchy`, `rows` (its default), `columns`, `tables`, `json-pretty`, and
   `json-compact`, refined by `--label`, `--header-level`, and `--table-shape`.
   `validate` and `info` still take `markdown` (default), `pretty-json`, or
   `compact-json`, and [`palette list`](palette/list.md) shares those values
   but adds a `hierarchy` tree like [`hierarchy show`](hierarchy/show.md) and
   defaults to it rather than `markdown`; they adopt the shared names in a
   later consistency pass. [`hierarchy show`](hierarchy/show.md) prints only
   its tree.
6. Multiple values are passed by repeating the flag, as in
   `--select-index 0 --select-index 3`, not as one comma-separated argument. The
   exception is the `--texture-map` channel list, where the comma-separated RGBA
   packing is a single structured value.

## Glob patterns

Patterns follow `.gitignore` rules, not grep substring matching. The
[`hierarchy show`](hierarchy/show.md) patterns and the `--select` path glob share
one rule set, matched by the `pathspec` engine:

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

[`mesh`](mesh.md) and [`material`](material.md) choose which objects to output,
and [`palette quantize`](palette/quantize.md) and
[`palette remap`](palette/remap.md) choose which objects to dither, with two
repeatable options, one per addressing mode, so a value is never parsed as
either an index or a glob. Selection targets objects; under `mesh` each matched
object is meshed as pure geometry with no hierarchy-node transform, so a path is
only the selection key, not placement.

1. `--select-index <index>`: an object index into the document's `objects`,
   a plain integer such as `0` or a range `a-b` such as `2-5`. Repeat the flag
   to pick several, as in `--select-index 0 --select-index 3`. Index is the
   canonical object reference in the spec.
2. `--select <glob>`: a glob over hierarchy paths, matched with the shared
   [glob rules](#glob-patterns) exactly as [`hierarchy show`](hierarchy/show.md)
   matches node paths. The candidates are the path of every node and every object
   it places: a node's path is the chain of node names from a root, an object's
   path that chain plus the object. A match selects every object at or under it,
   so matching a node selects its whole subtree, just as selecting a node in
   `hierarchy show` brings in its subtree, and matching an object selects that
   object. `--select a` selects every object under node `a`, `--select a/**` the
   same by its descendants, and `--select a/b` only object `b`. The graph is a
   DAG, so an object reached through several parents has one path per placement
   and matches when any path does; an object no node references has just its name
   as its path. Names are not unique, so a glob may match several objects.

Both options repeat, and every `--select-index` and `--select` value unions its
matches. Given neither, every object is selected. `material` outputs the
selection, and `quantize` and `remap` dither it; `mesh` outputs it too but for
now requires it to resolve to a single object (see [mesh](mesh.md)).

Baking a matched node's subtree and transforms into one larger placed mesh,
rather than selecting its objects as pure geometry, is a separate mode left for a
later pass.
