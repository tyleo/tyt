# voxrender-bench-desktop

voxrender-bench's box view as a desktop app on the high-performance adapter.
`view`, `view-check`, and `lit-check` each draw or check one view of a scene
with no window. `gallery` shows the models under a directory in a window.

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

`view` and `view-check` take `--detail` with the fewest pixels a voxel
spans. Each object then keeps coarser copies of itself, and the view draws a
place at the coarsest copy whose voxels span that many pixels or fewer:

```sh
cargo run --release -p voxrender-bench-desktop -- view-check S5 --detail 1.4
```

## Gallery

`gallery` opens a window on the Voxel Json models under a directory and
shows them one at a time, as the Quest app's gallery does:

```sh
cargo run --release -p voxrender-bench-desktop -- gallery <models-dir>
```

A model lands on a table before the head and spans a meter. A keyboard and
a mouse steer:

| Control | Effect |
| --- | --- |
| Right arrow | Shows the next model |
| Left arrow | Shows the model before |
| W, A, S, and D | Walk the way the head faces |
| E and Q | Rise and sink |
| A drag | Turns the head |
| Up arrow | Grows the model, which brings the head toward what it faces |
| Down arrow | Shrinks the model |
| Tab | Switches between the table and life size |
| Escape | Closes the window |

A model grows around what the middle of the window shows. The head walks
1.5 m a second in its space, which crosses a grown model more slowly.

The window's title holds the frames it draws a second and the slowest of
them. The app prints each model as it loads, and the frames it drew once
the window closes. The image keeps the pixels the window opens with and
stretches when the window's size changes.

These flags steer a run:

1. `--model` starts the gallery at the first model whose path holds the
   flag's text
2. `--detail` takes the fewest pixels a voxel spans, as `view` takes it
3. `--width` and `--height` set the size the window opens at, in the
   display's points
4. `--seconds` closes the window after that many seconds
