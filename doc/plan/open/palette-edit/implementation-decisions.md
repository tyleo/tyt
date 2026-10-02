# Implementation decisions

Code-level choices a reviewer of the Rust would want explained, recorded as
they land.

## S1. f64 language

- voxsmith checks the f32 range in `check_f32_range` and leaves the rounding
  to the glTF writer, which already narrows every float with `as f32`. The
  meshdoc keeps full `f64` values. Every material factor goes through the
  check, and only the unbounded `normalScale`, `emissiveStrength`,
  `alphaCutoff`, and `ior` can fail it. Custom float attributes go through it
  after the transfer. `COLOR_0` lies in `[0, 1]` and skips it.
- `CHROMA_FLOOR` stays at `1e-6`. With the `f32` narrowing gone, the rounded
  Oklab matrices leave at most about `1.7e-7` of chroma on a gray up to a
  linear `100`.
- `assert_close` keeps its `1e-5` relative tolerance because several tests
  compare against hand-typed reference values.
- Greedy merge class keys split an `f64` component's bits into two `u32`
  words.
- voxsurface's `corner_occlusion` and `mesh_occlusion` return `f64`, so the
  occlusion thirds reach the language and voxrender exact. Vertex positions
  stay `f32`.

## S2. Swatch count and mentioned names

- `Program::free_names` scans for the names the environment supplies, not
  every name the program mentions. It counts a name a binding reads before the
  program binds it, and a name an end-scope expression reads that the program
  never binds. A property the program rebinds before reading it stays unbound.
  Its value never blocks the run.
- `Destination::of_record` parses each expression and keeps it on the
  `Destination`. The run collects the free names from those expressions before
  it binds. A destination parse error now rises before the environment binds.
  `Destination` drops `Eq` because `Expression` does not implement it.
- Computed bindings bind even when nothing reads them because each comes from
  an explicit flag.
- `CheckedRecord` parses and checks a record once per object. A greedy mesh
  evaluates it twice, over the culled pre-pass and over the merged geometry.
  The property values bind once. The computed values and the groupings rebind
  for each geometry. `computed_type` gives a computed binding its type before
  any geometry exists. `Streams` now derives once per object.
- A destination check error now rises before an evaluation error.

## S3. voxcore setters

- `VoxValuePool` gains one `retain_<kind>_value` per kind, matching its
  constructors. `VoxMain` wraps each one under the same name. These replace
  the README's `retain_value_pool_value` and `VoxValuePoolValue`, so the
  caller picks the method for its value's kind. S5 compares a written value
  against a pool's values through `VoxValuePoolValueRef`.
- Each append checks the pool's kind first, then the value's domain.
- The checked constructors scan their own values before building and drop
  `checked`. `first_out_of_domain_value`, now only the audit `validate` runs,
  matches the kind once and walks the typed column. The domain checks take
  refs, so one function serves a constructor, an append, and the audit.
- The appends and `VoxPalette::set_value_id` are public, like `move_value` and
  `retain_property`. `VoxMain` adds the cross-reference checks.
- A rejected value has no id yet, so the two new errors carry none:
  `RetainedValueKind` and `MalformedRetainedValue`.
- An appended value can reuse a released id. It still lists last.
