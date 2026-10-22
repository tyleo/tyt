# sdfj-builder

Records a voxel model written in TypeScript as an SDF Json (`.sdfj`) document.
The builder puts the modeling API in scope, imports the model file, and writes
the steps and parts the model's default export lists. Each named export writes a
name. The document can then serve as a library.

```sh
node ts/main.ts chair.ts chair.sdfj
```

The builder runs under Node 24, Bun, and Deno with no dependencies. It reads the
libraries from `libraries.json` beside `main.ts`. The file holds an array of
each library's `name` and `.sdfj` `document`. `vars.json` beside `main.ts` holds
the build variables as an object of JSON values by name. `globals.d.ts` declares
the API for type-checking model files.

A call errors on an argument the document cannot hold. vxl checks the arguments'
values when it voxelizes. The tests run under `deno test`.

## Writing

The builder walks the default export depth first and writes each value into its
table after the values it references. The named exports follow in name order. A
value met again reuses its entry, so a shape feeding several steps appears once.
Every reference within a table therefore points to an earlier entry.

1. An entry's `kind` holds the name of the call that made it, and its other keys
   take the call's parameter and option names
2. An option the model leaves out stays out of the entry, and an empty table or
   names map stays out of the document
3. Each number takes its shortest round-trip form, and an index writes as a
   whole number
4. A part writes one node named after it and, when its list holds steps, one
   object of the same name. A part in several lists writes one node that each
   parent lists
5. The default export writes one root node named after the model file's stem,
   and the steps outside every part go in its object
6. Each named export writes a name, with each map in name order. An entry a
   model copies from a library keeps every name it holds there

The builder writes five shorthands in full:

1. A `set` of one point writes a list of one point
2. A `speckle` with one accent writes a list of one accent
3. A `scale` by one factor writes the factor once per axis
4. A `grain` with one `base` writes the materials of `shades(base)`
5. A `shades` call writes `count` even when the model leaves it out

The folder is also the `sdfj-builder` crate, with its Rust in `src/`.
`SDFJ_BUILDER_FILES` embeds the files a run needs. `JavaScriptRuntime` holds the
arguments that start the builder under each runtime.
