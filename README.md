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
tyt integration completion print zsh          # Print shell completions
```

Run `tyt <command> --help` for full details on any subcommand:

```
> tyt fbx --help
Works with FBX files

Usage: tyt fbx <command>

Commands:
  create-point-cloud  Creates a cloud of random points inside a mesh volume (or on the surface with `--surface`) within an FBX file
  extract             Extracts a single mesh matching a selection pattern from the input FBX file, unparents it keeping the world transform, deletes everything else, and renames the mesh object and its datablock to `output-mesh-name`. Exactly one mesh must match
  hierarchy           Prints the FBX object hierarchy as a box-glyph tree, showing each object's name and type
  modify              Applies mutating operations to matched objects in an FBX file
  reduce              Collapses all mesh objects in the input FBX file into a single joined mesh. Clears parenting while keeping world transforms, deletes now-unused empties, joins all meshes, and renames the result to `output-mesh-name`
  rename              Renames every object whose hierarchy path matches a selection pattern. The new name for each matched object is composed as `{prefix}{name-or-old}{suffix}{suffix-num}`, where any omitted piece is treated as empty and `name-or-old` is `--name` when set, otherwise the object's existing name. Matches any object type: `MESH`, `ARMATURE`, `EMPTY`, `LIGHT`, `CAMERA`, and so on
  render              Renders the meshes in an FBX file from a specified camera position. The result is written to an image file, displayed inline in the terminal (Kitty, iTerm2, Sixel, or ANSI fallback), or both
  transform           Overwrites individual position, rotation, and scale components on every object whose hierarchy path matches a selection pattern. Unset components are left untouched
  help                Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Shell completions

`tyt integration completion print <shell>` prints completions to stdout. Install the completions for your shell:

```sh
# Bash (bash-completion v2 user-local)
mkdir -p ~/.local/share/bash-completion/completions
tyt integration completion print bash > ~/.local/share/bash-completion/completions/tyt

# Zsh
mkdir -p ~/.zsh/completions
tyt integration completion print zsh > ~/.zsh/completions/_tyt
# Then ensure this is in your .zshrc *before* compinit:
#   fpath=("$HOME/.zsh/completions" $fpath)

# Zsh (Oh My Zsh)
mkdir -p ~/.oh-my-zsh/custom/completions
tyt integration completion print zsh > ~/.oh-my-zsh/custom/completions/_tyt
# If completions don't show up, ensure this is in your .zshrc *before* compinit:
#   fpath=("$HOME/.oh-my-zsh/custom/completions" $fpath)

# Fish
mkdir -p ~/.config/fish/completions
tyt integration completion print fish > ~/.config/fish/completions/tyt.fish

# PowerShell
# recommended: keep completions in a separate file and dot-source it from your $PROFILE
$dir = Join-Path $HOME ".config\powershell"
New-Item -ItemType Directory -Force -Path $dir | Out-Null

tyt integration completion print powershell | Set-Content -Encoding UTF8 (Join-Path $dir "tyt-completions.ps1")

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
| `ty-preferences`  | Loads `.tytconfig` preferences from user home and git root                |
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

## Development

Images and other binaries live in [Git LFS](https://git-lfs.com). The voxrender-cpu golden tests compile the real PNGs in, so install `git-lfs` before cloning. A clone made without it holds pointer files, and `git lfs pull` fetches them.

After cloning, run setup once:

```sh
npm run setup
```

This points `core.hooksPath` at `.githooks`, so a pre-commit hook runs `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` before each commit, and the Git LFS hooks upload binaries on push. A commit touching `projects/utilities/sdfj-builder` also runs the builder's Deno checks and needs `deno`. Setup also checks out the `submodules/branded-id` submodule, where `branded-id` is developed. The workspace builds against the published `branded-id`. To build against unreleased submodule changes, add `branded-id = { path = "submodules/branded-id" }` under the root manifest's `[patch.crates-io]`, and drop it once that version is published. Without npm, run `git config core.hooksPath .githooks` and `git submodule update --init submodules/branded-id`.

Format and lint manually with:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

### Quest builds

`voxrender-bench` also builds as a Meta Quest 3 app. Gradle and cargo-ndk build it on macOS with the tools below. Each step follows its tool's recommended install. The bench's README holds the build and run commands. Gradle needs no install because the bench checks in its wrapper.

1. A JDK starts Gradle. Gradle then fetches the JDK its daemon pins. Adoptium recommends a package manager:

   ```sh
   brew install --cask temurin
   ```

2. The Rust Android target and cargo-ndk:

   ```sh
   rustup target add aarch64-linux-android
   cargo install cargo-ndk --locked
   ```

3. Google's Android CLI, which replaces the deprecated `sdkmanager`:

   ```sh
   brew tap android/tap
   brew install android-cli
   ```

4. Add these lines to your login shell's environment, then open a new shell:

   ```sh
   export JAVA_HOME="$(/usr/libexec/java_home)"
   export ANDROID_HOME="$HOME/Library/Android/sdk"
   export PATH="$ANDROID_HOME/platform-tools:$PATH"
   ```

   `platform-tools` comes first in `PATH` so no other `adb` shadows it.

5. The SDK packages, at the versions the Gradle build pins:

   ```sh
   android --sdk="$ANDROID_HOME" sdk install platform-tools platforms/android-34 build-tools/36.0.0 ndk/30.0.16248370
   ```

6. `which -a adb` lists only `$ANDROID_HOME/platform-tools/adb`. A second `adb`, such as Homebrew's `android-platform-tools` cask, runs its own server on the same port. Meta Quest Developer Hub bundles another, so point its ADB Path setting at the SDK's `adb`.

7. Developer mode on the headset:
   1. At [developers.meta.com](https://developers.meta.com), create or join a team, then verify the account by SMS or a payment method
   2. In the Meta Horizon phone app, open the headset's settings and turn on Developer Mode
   3. Connect the headset by USB-C and allow USB debugging from this computer
   4. `adb devices` lists the headset

## Releasing

A release publishes the crates you name and only the workspace crates they need. `scripts/release.sh` picks them and needs `jq`, `perl`, and `curl`.

`branded-id` lives outside the workspace in its submodule, so the script skips it. Publish a new version from its own repository first when the release needs it, and drop the workspace's `[patch.crates-io]` entry for it.

1. Print the plan for the crates to release:

   ```sh
   scripts/release.sh plan tyt vxl
   ```

   The plan walks the named crates' workspace dependencies and bumps three kinds of crate: the named ones, the ones whose packaged files changed since their `crate@version` tag, and the ones that depend on a crate taking a breaking bump. A crate without a tag is new and ships at its current version

2. Pick the bumps. Each bump is breaking by default, taking `0.2.3` to `0.3.0`, since cargo treats `0.2.3` to `0.2.4` as compatible. List the crates whose changes keep their API, such as documentation fixes, with `--patch`. A patch bump leaves its dependents alone, so the plan shrinks:

   ```sh
   scripts/release.sh plan --patch sdfcore,voxj tyt vxl
   ```

3. Apply the plan. The script sets the versions, raises every requirement on a crate taking a breaking bump, refreshes the lockfile, and fails if a workspace crate still resolves from crates.io:

   ```sh
   scripts/release.sh bump --patch sdfcore,voxj tyt vxl
   ```

4. Run the tests, commit, tag each new `crate@version`, and push:

   ```sh
   cargo test --workspace
   git commit -am "chore(release): bump the changed crates"
   cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | "\(.name)@\(.version)"' | while read tag; do git rev-parse -q --verify "refs/tags/$tag" >/dev/null || git tag "$tag"; done
   git push origin main $(git tag --points-at HEAD | sed 's#^#refs/tags/#')
   ```

5. Publish the named crates and the dependencies crates.io lacks, one at a time in dependency order. `--dry-run` prints the order first:

   ```sh
   scripts/release.sh publish --dry-run tyt vxl
   scripts/release.sh publish tyt vxl
   ```

   crates.io accepts a burst of about 30 uploads, then about one a minute. The script waits out a `429 Too Many Requests` and retries. It skips published versions, so rerun it after any other failure

## License

[MIT](LICENSE)
