# voxrender-bench-desktop

voxrender-bench's box view as a headless desktop app. Each subcommand draws
or checks one view of a scene on the high-performance adapter.

## Run

```sh
cargo run --release -p voxrender-bench-desktop -- view S1 s1.png
cargo run --release -p voxrender-bench-desktop -- view-check S1
cargo run --release -p voxrender-bench-desktop -- lit-check S1
```

`view` draws a bench scene's first view as the shaded box raster and writes
it as a PNG. The view takes the image's shape. `view-check` draws that view
and the reference's image of it over a square image and prints how many of
the pixels that hit something opaque differ. `lit-check`
fills a scene's corner shadows on the GPU as the view does and again on the
CPU through the reference's walk. It prints how many corners differ. All three
take the start of a bench scene's name, such as `S1`.

Each of the three also takes the path of a Voxel Json model in place of a
scene's name. A model draws from vxl's hero view under vxl's studio rig and
the bloom vxl's review renders take:

```sh
cargo run --release -p voxrender-bench-desktop -- view chair.voxjz chair.png
```

`view` and `view-check` look from another point with `--eye` and
`--look-at`. Each flag takes three coordinates in the scene's meters. The
lights stay where the first view put them:

```sh
cargo run --release -p voxrender-bench-desktop -- view-check tank.voxjz --eye 0.1 0.08 0.7 --look-at 0 0.05 0
```

`view-check --headset` looks through a Quest 3's two eyes in place of the
view. The eyes sit 64 mm apart about the view's position and draw in one
frame. Each eye's edges sit off its axis as the headset gives them. The
command prints a line an eye:

```sh
cargo run --release -p voxrender-bench-desktop -- view-check S1 --headset
```

A view holds a scene's tiles in pages, each as large as one of the device's
storage bindings. `view` and `view-check` take `--page-bytes` to set a
smaller page, which draws a small scene in several:

```sh
cargo run --release -p voxrender-bench-desktop -- view S1 room.png --page-bytes 1400000
```
