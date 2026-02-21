# tyt - Tyleo's Tools

My command-line tools and a single app, `tyt`, that ties them all together. It aggregates specialized utilities for working with files, images, 3D models, material textures, and more into one binary.

## Install

```sh
cargo install tyt --features bin
```

For local development:

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

Some examples:

```sh
tyt cubemap faces-to-equirect skybox          # Stitch cube faces into a panorama
tyt fbx hierarchy model.fbx                   # Print the object hierarchy of an FBX file
tyt fs find "*.png"                           # Find files with .gitignore style patterns
tyt image pixelate input.png 8                # Pixelate an image
tyt material create-mse out --prefix my-tex   # Pack an MSE texture from material maps
tyt completion zsh                            # Generate shell completions
```

Run `tyt <command> --help` for full details on any subcommand:

```
> tyt fbx --help
Operations on FBX files

Usage: tyt fbx <command>

Commands:
  create-point-cloud  Creates a cloud of random points on a mesh surface within an FBX file
  extract             Extracts the first direct child mesh under `parent_mesh_name` from the input FBX file, unparents it, keeping the world transform, and deletes everything else so the file only contains the extracted mesh. Finally, renames the mesh object and its datablock to `output-mesh-name`
  hierarchy           Prints the FBX object hierarchy as a tree with box-drawing glyphs, showing each object's name and type
  reduce              Collapses all mesh objects in the input FBX into a single joined mesh. Clears parenting while keeping world transforms, deletes now-unused empties, joins all meshes, and renames the result to `output-mesh-name`
  rename              Renames mesh objects and their datablocks in the input FBX file. If exactly one mesh exists it is renamed to `output-mesh-name`; if multiple exist they are renamed to `output-mesh-name`-001, -002, etc
  help                Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Shell completions

`tyt completion <shell>` prints completions to stdout. Install them for your shell:

```sh
# Bash (bash-completion v2 user-local)
mkdir -p ~/.local/share/bash-completion/completions
tyt completion bash > ~/.local/share/bash-completion/completions/tyt

# Zsh
mkdir -p ~/.zsh/completions
tyt completion zsh > ~/.zsh/completions/_tyt
# Then ensure this is in your .zshrc *before* compinit:
#   fpath=("$HOME/.zsh/completions" $fpath)

# Zsh (Oh My Zsh)
mkdir -p ~/.oh-my-zsh/custom/completions
tyt completion zsh > ~/.oh-my-zsh/custom/completions/_tyt
# If completions don't show up, ensure this is in your .zshrc *before* compinit:
#   fpath=("$HOME/.oh-my-zsh/custom/completions" $fpath)

# Fish
mkdir -p ~/.config/fish/completions
tyt completion fish > ~/.config/fish/completions/tyt.fish

# PowerShell
# recommended: keep completions in a separate file and dot-source it from your $PROFILE
$dir = Join-Path $HOME ".config\powershell"
New-Item -ItemType Directory -Force -Path $dir | Out-Null

tyt completion powershell | Set-Content -Encoding UTF8 (Join-Path $dir "tyt-completions.ps1")

if (!(Test-Path $PROFILE)) { New-Item -ItemType File -Force -Path $PROFILE | Out-Null }
$line = ". `"$dir\tyt-completions.ps1`""
if (-not (Select-String -Quiet -Path $PROFILE -Pattern [regex]::Escape($line))) {
  Add-Content -Path $PROFILE -Value $line
}
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
cargo check                         # Type-check the workspace
cargo build -p tyt --features bin   # Build the binary
```

## License

[MIT](LICENSE)
