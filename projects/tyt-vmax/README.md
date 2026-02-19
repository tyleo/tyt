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
vmax completion zsh                                            # Generate shell completions
```

Run `vmax <command> --help` for full details on any subcommand:

```
> vmax --help
Usage: vmax <command>

Commands:
  completion   Generate shell completions
  hierarchy    Prints the Voxel Max hierarchy
  pack         Packs a .vmax directory by stripping history files
  rename-node  Renames nodes in the Voxel Max scene hierarchy matching a glob pattern
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
