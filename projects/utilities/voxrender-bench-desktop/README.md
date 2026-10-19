# voxrender-bench-desktop

voxrender's GPU benchmarks as a headless desktop app. Each subcommand runs one
benchmark from `voxrender-bench` on the high-performance adapter and prints
its report in Markdown.

## Run

```sh
cargo run --release -p voxrender-bench-desktop -- b1 --size 256
cargo run --release -p voxrender-bench-desktop -- b1-times
```

`b1-times` waits for each submit to finish before the next, as a display's
frames do. It needs an adapter with timestamp queries.
