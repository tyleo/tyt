# tyt - Tyleo's Tools

My command-line tools and a single app, `tyt`, that ties them all together. It aggregates specialized utilities for working with files, images, 3D models, material textures, and more into one binary.

## Install

```sh
npm run install:tyt
```

### External dependencies

Some commands shell out to external tools. Install the ones you need:

| Tool                                                    | Used by                        |
| ------------------------------------------------------- | ------------------------------ |
| [Blender](https://www.blender.org/)                     | `fbx`                          |
| [FFmpeg](https://ffmpeg.org/)                           | `cubemap`                      |
| [ImageMagick](https://imagemagick.org/) (`magick`)      | `cubemap`, `image`, `material` |
| [ripgrep](https://github.com/BurntSushi/ripgrep) (`rg`) | `fs`                           |

## Usage

```
tyt <command> <subcommand> [options]
```

```sh
tyt cubemap faces-to-equirect skybox          # Stitch cube faces into a panorama
tyt fbx hierarchy model.fbx                   # Print the object hierarchy of an FBX file
tyt fs find "*.png"                           # Find files with .gitignore style patterns
tyt image pixelate input.png 8                # Pixelate an image
tyt material create-mse out --prefix my-tex   # Pack an MSE texture from material maps
tyt completion zsh                            # Generate shell completions
```

Run `tyt <command> --help` for full details on any subcommand.

### Shell completions

`tyt completion <shell>` prints completions to stdout. Install them for your shell:

```sh
# Bash
tyt completion bash > ~/.local/share/bash-completion/completions/tyt

# Zsh (add the completions directory to your fpath in .zshrc if needed)
tyt completion zsh > ~/.zfunc/_tyt

# Fish
tyt completion fish > ~/.config/fish/completions/tyt.fish

# PowerShell (add to your $PROFILE)
tyt completion powershell >> $PROFILE
```

## Configuration

`tyt` reads preferences from `.tytconfig` files in two locations:

- `~/.tytconfig` - User-level preferences
- `<git-root>/.tytconfig` - Repository-level preferences

Used by some commands to configure behavior.

## Project structure

This is a Cargo workspace. Each crate lives under [`projects/`](projects/) with its own README. The command crates (`tyt-cubemap`, `tyt-fbx`, etc.) each define a `Dependencies` trait and a feature-gated `DependenciesImpl`, and the root `tyt` crate ties them all together.

A few shared crates support the architecture:

| Crate             | Description                                                               |
| ----------------- | ------------------------------------------------------------------------- |
| `tyt-common`      | Shared types (e.g. `ExecFailed`) used across all crates                   |
| `tyt-injection`   | Free-function helpers used by `DependenciesImpl`s (behind `impl` feature) |
| `tyt-preferences` | Loads `.tytconfig` preferences from user home and git root                |
| `ty-math`         | Math types shared across crates                                           |
| `ty-math-serde`   | Serde support for `ty-math` types                                         |
| `tyt-meta`        | Scaffolding tools for adding new crates and commands (see below)          |

### Adding new crates and commands with `tyt-meta`

`tyt meta create-command` scaffolds new sub-crates and commands. Without `--parent` it creates an entirely new `tyt-<name>` crate with all the boilerplate (`Cargo.toml`, `Dependencies` trait, error types, etc.) and wires it into the workspace and top-level binary automatically. With `--parent` it adds a command to an existing crate.

```sh
# Create a brand-new tyt-audio crate
tyt meta create-command Audio audio "Operations on audio files."

# Add a command to an existing crate
tyt meta create-command Normalize normalize "Normalize audio levels." --parent audio
```

## Building from source

```sh
cargo check                     # Type-check the workspace
cargo build -p tyt --features bin   # Build the binary
```

## License

[MIT](LICENSE)
