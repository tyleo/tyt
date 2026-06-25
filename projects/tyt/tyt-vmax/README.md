# tyt-vmax

Commands for working with [Voxel Max](https://www.voxelmax.com/) `.vmax` directories.

## Usage

```
vmax <command> [options]
```

Some examples:

```sh
vmax hierarchy my-scene.vmax                                   # Print the scene hierarchy
vmax pack my-scene.vmax                                        # Strip history files in-place
vmax pack my-scene.vmax --output-vmax packed.vmax              # Strip history into a copy
vmax rename-node my-scene.vmax "Cube*" "Box"                   # Rename matching nodes
vmax to-voxj my-scene.vmax > my-scene.voxj                     # Convert to Voxel Json
vmax to-voxj my-scene.vmax --format zip > my-scene.voxjz       # Convert to compressed Voxel Json
vmax to-voxj my-scene.vmax --optimize size > my-scene.voxj     # Pick the smallest encodings
vmax completion zsh                                            # Generate shell completions
```

`to-voxj` writes the document to stdout. `--format` selects the form (`json`,
`zip`, or `pretty`); `--optimize` (`size`/`fast`/`pretty`) picks the block
encodings automatically, or set them explicitly with `--position-encoding` and
`--sample-encoding`.

Run `vmax <command> --help` for full details on any subcommand:

```
> vmax --help
Usage: vmax <command>

Commands:
  completion   Generate shell completions
  hierarchy    Prints the Voxel Max hierarchy
  pack         Packs a .vmax directory by stripping history files
  rename-node  Renames nodes in the Voxel Max scene hierarchy matching a glob pattern
  to-voxj      Converts a .vmax package to a Voxel Json (.voxj / .voxjz) document
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Building from source

```sh
cargo check                              # Type-check the workspace
cargo build -p tyt-vmax --features bin   # Build the binary
```

## License

[MIT](LICENSE)
