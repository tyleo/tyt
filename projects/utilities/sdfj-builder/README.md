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

The folder is also the `sdfj-builder` crate, with its Rust in `src/`.
`SDFJ_BUILDER_FILES` embeds the files a run needs. `JavaScriptRuntime` holds the
arguments that start the builder under each runtime.
