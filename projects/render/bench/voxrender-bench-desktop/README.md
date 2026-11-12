# voxrender-bench-desktop

voxrender's GPU benchmarks as a headless desktop app. Each subcommand runs one
benchmark from `voxrender-bench` on the high-performance adapter and prints
its report in Markdown.

## Run

```sh
cargo run --release -p voxrender-bench-desktop -- b1 --size 256
cargo run --release -p voxrender-bench-desktop -- b1-times
cargo run --release -p voxrender-bench-desktop -- b2-times
cargo run --release -p voxrender-bench-desktop -- view S1 s1.png
cargo run --release -p voxrender-bench-desktop -- view-check S1
cargo run --release -p voxrender-bench-desktop -- lit-check S1
```

`b1-times` and `b2-times` wait for each submit to finish before the next, as
a display's frames do. They need an adapter with timestamp queries. `b1` and
`b2-times` also need ray queries. `b2-times` leaves out a raster arm whose
multiview or multisampled arrays the adapter lacks.

`view` draws a bench scene's first view as the shaded box raster and writes
it as a PNG. `view-check` draws that view and the reference's image of it and
prints how many of the pixels that hit something opaque differ. `lit-check`
fills a scene's corner shadows on the GPU as the view does and again on the
CPU through the reference's walk. It prints how many corners differ. All three
take the start of a scene's name in B1's report.
