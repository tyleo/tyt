# sdfj-builder

Records a voxel model written in TypeScript as an SDF Json (`.sdfj`) document.
The [TypeScript API](docs/typescript-api.md) lists every call a model file can
make. The [sdfj format](../../sdf-formats/sdfj/docs/sdf-json-file-format.md)
sets what each entry of the document computes.

```sh
node ts/main.ts chair.ts chair.sdfj
```

The builder puts the TypeScript API in scope and imports the model file. Each
call returns a value that records its arguments, and the model's default export
lists the steps and parts those values build. The builder then writes every
value the exports reach as an entry of the document.

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

## Errors

A call errors on an argument the document cannot hold, and the error shows the
call, the argument, and the value. The builder checks:

1. The default export is an array of steps and parts
2. Each argument has the type the TypeScript API declares, and an options object
   holds only its call's options
3. Numbers are finite, and `json` holds only JSON values
4. `lib` and `mat` read only names the libraries hold
5. `shades` takes a whole count above zero, and a boolean takes at least one
   shape
6. Each named export holds a material, a pattern, a shape, a step, a part, or a
   function, and no two different entries of one kind share a name
7. A library part's node holds at most one object, which takes the node's name,
   lists at least one step, and belongs to no other node

The sdfj format's validation checks the values the document holds.

## Running

The builder runs under Node 24, Bun, and Deno with no dependencies. It reads
three files beside `main.ts`:

1. `libraries.json` holds an array of each library's `name` and `.sdfj`
   `document`
2. `fonts.json` holds the `.sdfj` `fonts` entries a `text` can read
3. `vars.json` holds the build variables as an object of JSON values by name

`globals.d.ts` declares the API for type-checking model files. The tests run
under `deno test`.

The folder is also the `sdfj-builder` crate, with its Rust in `src/`.
`SDFJ_BUILDER_FILES` embeds the files a run needs, and
`SDFJ_BUILDER_TYPESCRIPT_API` embeds the TypeScript API. `JavaScriptRuntime`
holds the arguments that start the builder under each runtime.
