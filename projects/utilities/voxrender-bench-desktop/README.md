# voxrender-bench-desktop

voxrender's GPU benchmarks as a headless desktop app. Each subcommand runs one
benchmark from `voxrender-bench` on the high-performance adapter and prints
its report in Markdown.

## Run

```sh
cargo run --release -p voxrender-bench-desktop -- b1 --size 256
```
