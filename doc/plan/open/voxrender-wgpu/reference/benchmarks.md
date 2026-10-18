# voxrender-wgpu benchmarks

_Part of the [voxrender-wgpu plan](../README.md)._ Each run gets a section with
its results and its device, OS and driver versions, GPU level, and
temperature. The [research](research.md)'s section 7 sets each benchmark's
threshold.

## B1 desktop, 2026-10-18

The integer walk's ids and corner shadow bits on this Mac, against the
reference. The run times nothing, so it has no GPU level or temperature.

- Device: Apple M4 Max, Metal 4, macOS 27.0.1 (26A434)
- wgpu 30.0.1 on Metal. The adapter reports no driver version
- Command: `cargo run --release -p voxrender-bench -- b1 --size 256`
- Samples: each pixel's placement, cell, and face. On a pixel whose ids
  match, each light's four corner shadow rays, as whether each meets a
  surface
- Float walk: an f32 DDA over the same quantized inputs and tie order

| Scene | Views | Pixels | Shadow samples | Id mismatches | Shadow mismatches | Float id flips | Float shadow flips |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| cube | 1 | 65536 | 55176 | 0 | 0 | 0 | 0 |
| glass | 1 | 65536 | 128188 | 0 | 0 | 1 | 0 |
| glow | 1 | 65536 | 105484 | 0 | 0 | 0 | 0 |
| l-shape | 1 | 65536 | 89796 | 0 | 0 | 1 | 0 |
| room | 1 | 65536 | 133384 | 0 | 0 | 0 | 0 |
| spot-room | 1 | 65536 | 133384 | 0 | 0 | 0 | 0 |
| two-placements | 1 | 65536 | 57504 | 0 | 0 | 0 | 0 |
| precision views | 10 | 655360 | 0 | 0 | 0 | 19 | 0 |
| precision shadows, sparse | 3 | 196608 | 7031760 | 0 | 0 | 2 | 0 |
| precision shadows, dense | 3 | 196608 | 7186560 | 0 | 0 | 1 | 0 |
| precision shadows, stairs | 3 | 196608 | 6026240 | 0 | 0 | 0 | 0 |

The desktop half passes B1's line. The integer walk matches the reference on
every sample, including the shadow starts that two-placements carries into
the other grid in f32. A run at 64 by 64 pixels also matches everywhere.

The float walk flips 24 of 1,572,864 pixels and no shadow sample. The
research's simulation found f32 against f64 disagreeing on 2.3 to 8.0% of
corner samples. It traced that to input rounding, which the shared
quantization now removes for both walks. Every float flip is on a primary
ray.
