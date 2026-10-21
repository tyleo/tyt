# voxrender-bench-desktop

voxrender's GPU benchmarks as a headless desktop app. Each subcommand runs one
benchmark from `voxrender-bench` on the high-performance adapter and prints
its report in Markdown.

## Run

```sh
cargo run --release -p voxrender-bench-desktop -- b1 --size 256
cargo run --release -p voxrender-bench-desktop -- b1-times
cargo run --release -p voxrender-bench-desktop -- b2-times
```

`b1-times` and `b2-times` wait for each submit to finish before the next, as
a display's frames do. They need an adapter with timestamp queries.
