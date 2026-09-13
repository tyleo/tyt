# vox-value-language

A typed expression language over per-swatch, per-voxel, per-face, and
per-corner values.

A program is a list of bindings, `name = expr;`, evaluated in order. The names
a program reads but never defines come from an environment the caller supplies,
and every binding's type and value come back out by name. The crate runs a
one-way pipeline over whole programs: `parse` takes the text to a syntax tree,
`check` settles every binding's type, and `eval` computes every binding's
value. The expression siblings `parse_expression`, `check_expression`, and
`eval_expression` run one expression in a program's end scope.

## Programs

A program is a sequence of statements, each a binding `name = expr` terminated
by `;`. A statement binds and does nothing else: a bare expression errors. The
empty statement, a lone `;`, is legal, so a program of several fragments joins
with a `;` after each. Bindings evaluate in order, and a name can be redefined
let-style: the right side reads the bindings visible at that point, so
`roughness = pow(roughness, 2)` reads the earlier value and rebinds the name.
There is no recursion.

```
tint = baseColor.rgb;
dim = tint * 0.5; bright = tint * 1.2;
```

## Shapes and domains

A value is plain or an array, and an array has a domain, what it holds one
entry per. The domains form a ladder, bottom to top:

1. `plain`: a single entry
2. `swatch`: one entry per swatch
3. `voxel`: one entry per solid voxel
4. `face`: one entry per emitted face
5. `corner`: one entry per face corner, exactly four per face

A literal is plain. A name has the shape its definition or the environment
gives it. Elementwise operations pair arrays element by element and broadcast a
plain value across an array, so `1 - roughness` is an array wherever
`roughness` is. Two arrays of one domain always align, and mixed domains climb:
a plain value broadcasts everywhere, a swatch value reads per voxel through the
voxel's swatch, a voxel value reads per face through the face's voxels, and a
face value duplicates onto its four corners. The climb is implicit because
nothing is lost. `swatch(e)`, `voxel(e)`, `face(e)`, and `corner(e)` name the
same climb where the domain is the point: each lifts a value to the named
domain, is the identity on a value already there, and errors on a step down. A
climb moves entries and never touches them, so it carries a bool or a string as
readily as a number.

A step down loses entries, so it takes an explicit reduction naming its
destination; see [Reductions](#reductions).

`e[i]` samples an array at entry `i` into a plain value. The index is a plain
unsigned vec1 below the array's entry count; an `f32` index, an out-of-range
index, or an array index errors. Indexing and swizzling commute:
`color[0].rgb` and `color.rgb[0]` name the same value.

## Numbers

A number has a dimension, vec1 through vec4, and one of four types: `f32` and
the unsigned `u8`, `u16`, and `u32`. The types never mix, and nothing converts
implicitly: every operator, comparison, and function takes one numeric type
across its numeric operands, so `position.y * 0.5` errors where `position` is
`u32` and `f32(position.y) * 0.5` converts.

A literal names its type or takes it from context. A decimal point makes an
`f32`, a suffix pins any type (`2f32`, `2u8`, `2u16`, `2u32`), and a bare whole
number takes the type of the operands beside it, so `mod(position.y, 2)` reads
`2` as `u32`. The functions that take `f32` alone type their bare literals as
`f32`, so `rgb(1, 1, 1)` is an `f32` triple. A bare index literal reads as
`u32`. A literal nothing types errors, and a suffix fixes it: `x = 1` errors
where `x = 1u32` does not.

The conversions are explicit and componentwise:

- `f32(e)` takes an unsigned value exactly, erroring where `f32` holds no exact
  image of it.
- `u8(e)`, `u16(e)`, and `u32(e)` widen an unsigned value losslessly, narrow
  one under a range check, and take an `f32` only at exact whole components,
  erroring on a fraction rather than rounding.
- `ceil_u8` through `round_u32` round an `f32` by the named mode into the named
  range, erroring where the rounded value falls outside it.

Arithmetic keeps its type. Unsigned `+`, `-`, and `*` error on overflow and on
a difference below zero rather than wrapping, `/` floors with the floored `mod`
completing it, and unary `-` takes `f32` alone. `min`, `max`, and the sums keep
the operand type; every average returns an `f32`. A non-finite `f32` result,
NaN or infinity as from `0 / 0`, errors wherever it appears, and nothing
clamps: a bound is always the author's `clamp`.

## Booleans

A comparison makes a bool: `<`, `<=`, `>`, `>=`, `==`, and `!=` take a vec1 on
each side and yield one. `true` and `false` name a plain bool directly. `==`
and `!=` also compare two strings by value.

A wider comparison names its fold: inside `any(c)` and `all(c)` the sides share
a dimension or either is a vec1, the components compare one by one, and the
reduction folds the answers, `any` with or and `all` with and. The comparison
is legal only directly inside its reduction: a bare `vec3 < vec3` errors, and
the component answers never escape as a value. No bool vector exists.

`!`, `&&`, `^`, and `||` combine bools, with `^` the exclusive or. A bool never
mixes with a number: arithmetic on a bool errors, and every function rejects
one except `mix` and the domain climbs. `mix(x, y, cond)` picks `x` or `y` per
entry by the bool, so `mix(0f32, 1, glowing)` makes a `0`/`1` mask.

Shape follows the numeric rules: a comparison against a plain value broadcasts
across an array, two arrays pair element by element, and the logical operators
do the same. `==` and `!=` compare `f32` exactly.

## Strings

A string literal is double-quoted, `"MASK"`, and takes any characters except
the quote, with no escapes. A string has no components, no swizzle, no
arithmetic, and no coercion, so a string never meets a number or a bool.

Five operations touch the type. `==` and `!=` compare two strings into a bool,
entry by entry, with a plain side broadcasting across an array. `mix(x, y,
cond)` picks between two strings by a bool. `e[i]` samples a string array at an
entry. `default(name, fallback)` fills a missing string name. The domain climbs
lift a string's entries untouched.

```
glass = tag == "glass";
mode = mix("OPAQUE", "BLEND", min(color.a) < 1);
```

## Color spaces

Every expression evaluates in linear RGB, and the conversion functions visit
other spaces as plain vec3 math: the language never tracks which space a vec3
sits in. Linear RGB is the hub every space converts to and from, so a hop
between two others is two calls. The constructor names carry dimension, not
meaning, `rgb(...)` assembling an Oklab triple as readily as a color, and such
components read best through the position alphabet, `lab.x` rather than
`lab.r`.

`oklabFromRgb(c)` and `rgbFromOklab(l)` visit Oklab, the perceptual space, where
`.x` holds perceived lightness, 0 black to 1 white, `.y` runs green to red, and
`.z` blue to yellow. `distance` there measures how different two colors look.

`oklchFromRgb(c)` and `rgbFromOklch(l)` visit Oklch, Oklab's polar form: `.x`
holds the same lightness, `.y` chroma, 0 at gray, and `.z` hue as a turn in
`[0, 1]`. A gray has no hue, so `oklchFromRgb` answers hue 0 at zero chroma.
`rgbFromOklch` errors on a hue outside `[0, 1]`, leaving the wrap to the
author's `mod(h, 1)`, and on a negative chroma. A converted-back color can leave
the gamut, and no conversion clamps.

```
lab = oklabFromRgb(color.rgb);
reddish = distance(lab, oklabFromRgb(rgb(1, 0, 0))) < 0.25;
darker = rgbFromOklab(lab * rgb(0.8, 1, 1));
shifted = rgbFromOklch(oklchFromRgb(color.rgb) + rgb(0, 0, 0.1));
```

## Operators

From tightest to loosest: postfix (swizzle, index), unary `-` and `!`, `* /`,
`+ -`, the comparisons `< <= > >= == !=`, `&&`, `^`, `||`. The binary
operators associate left to right, so `a + 1 > b && c > d` reads as
`((a + 1) > b) && (c > d)`. Postfixes chain left to right. Unary minus nests,
so `- -x` is valid; there is no `--` token. Exponent is written `pow()`; `^` is
the exclusive or.

The dimension rules for the arithmetic operators: `+` and `-` take equal
dimensions; `*` takes equal dimensions or a vec1 on either side; `/` takes equal
dimensions or a vec1 divisor. The result takes the larger dimension.

## Swizzles

A swizzle is any sequence of 1 to 4 components, repeats allowed, where every
component exists in the source. Two alphabets name the same components, color
`rgba` and position `xyzw`, and one swizzle draws from one alphabet, so `v.xg`
errors. `r`/`x` always work, `g`/`y` need a vec2 or wider, `b`/`z` a vec3 or
wider, and `a`/`w` a vec4. Every dimension can be swizzled, vec1 included:
`s.r` is the identity, and repeats splat upward, so `s.rr` is a vec2. Results
can be wider or narrower than the source.

```
color.rgb     # vec4 to vec3, dropping alpha
orm.g         # one channel
0.5.rrr       # a grey vec3 splat from one number
tint.rrgg     # wider than its source
offset.xyz    # the position alphabet, the same value as .rgb
```

The dot and its member hug the source's final token, and an index bracket does
the same: `v.rg` and `tint[0]` are postfixes; `v .rg`, `v. rg`, and `tint [0]`
are errors.

## Functions

The dimension rules use `dim(e)` for an expression's dimension. Every function
below takes `f32` alone on its numeric arguments unless its entry says
otherwise, and every one pairs arrays element by element and broadcasts plain
values, the result an array when any argument is.

1. `r(x)`, `rg(x, y)`, `rgb(x, y, z)`, `rgba(x, y, z, w)` build a vector from
   vec1 parts.
2. `min(e)`, `max(e)`, `sum(e)`, `avg(e)` reduce an array across its whole
   domain, per component, to a plain value. `min`, `max`, and `sum` keep the
   operand type and take any; `avg` takes any and returns `f32`.
3. `min(a, b)`, `max(a, b)` are elementwise, a vec1 broadcasting from either
   side, any numeric type kept.
4. `abs(e)` is the componentwise magnitude.
5. `dot(a, b)`: `dim(a) = dim(b)`, nothing broadcasts; result vec1.
6. `length(e)`: the vector's magnitude; result vec1.
7. `distance(a, b)`: `length(a - b)`; `dim(a) = dim(b)`; result vec1.
8. `normalize(e)`: `e / length(e)`; a zero vector errors.
9. `cross(a, b)`: both vec3; result vec3.
10. `oklabFromRgb(c)`, `rgbFromOklab(l)`, `oklchFromRgb(c)`, `rgbFromOklch(l)`:
    vec3 in, vec3 out.
11. `pow(a, b)`: componentwise exponent; `dim(b) = dim(a)` or 1; result
    `dim(a)`.
12. `mod(a, b)`: the floored remainder `a - b * floor(a / b)`; `dim(b) =
    dim(a)` or 1; any numeric type kept; `mod(a, 0)` errors.
13. `clamp(x, lo, hi)`: pins each component into `[lo, hi]`; `dim(lo) =
    dim(hi)`, equal to `dim(x)` or 1; a component with `lo > hi` errors.
14. `lerp(a, b, t)`: `a + (b - a) * t`, unrestricted in `t`; `dim(a) = dim(b)`;
    `dim(t) = dim(a)` or 1.
15. `step(edge, x)`: 0 where `x < edge` and 1 elsewhere; `dim(edge) = dim(x)`
    or 1; result `dim(x)`.
16. `smoothstep(lo, hi, x)`: the Hermite ramp, 0 at `lo`, 1 at `hi`, held flat
    outside; `dim(lo) = dim(hi)`, equal to `dim(x)` or 1; a component with
    `lo >= hi` errors.
17. `floor(e)`, `ceil(e)`, `round(e)`: snap each component down, up, or to the
    nearest with halves away from zero; `f32` in and out.
18. `f32(e)`, `u8(e)`, `u16(e)`, `u32(e)`, and `ceil_u8` through `round_u32`:
    the conversions, componentwise, under the rules in [Numbers](#numbers).
19. `mix(x, y, cond)`: `x` where the bool is false and `y` where it is true.
    The branches share a dimension or are both strings.
20. `any(c)`, `all(c)`: fold a comparison's component answers into one bool
    per entry, `any` with or and `all` with and. The argument is a comparison
    written in place.
21. `faceAvg(e)`, `faceMin(e)`, `faceMax(e)`, `faceSum(e)`: a corner array to
    a face array.
22. `voxelAvg(e)`, `voxelMin(e)`, `voxelMax(e)`, `voxelSum(e)`: a face or
    corner array to a voxel array.
23. `swatchAvg(e)`, `swatchMin(e)`, `swatchMax(e)`, `swatchSum(e)`: a voxel,
    face, or corner array to a swatch array.
24. `swatch(e)`, `voxel(e)`, `face(e)`, `corner(e)`: the explicit climbs, any
    type carried.
25. `default(name, fallback)`: `name` where it has a value and `fallback` where
    the name is unbound. `name` is bare or backtick-quoted, and `fallback` is
    any `f32` or string expression of the same dimension.

The function set stays small: `sqrt(x)` is `pow(x, 0.5)`, `fract(x)` is
`mod(x, 1)`, and a signed remap is `n * 0.5 + 0.5`.

## Reductions

A step down the ladder takes a reduction naming its destination, and the
reduction accepts any array above it:

- `faceAvg` and its siblings take a corner array to a face array, reducing each
  face's four corners per component.
- `voxelAvg` and its siblings take a face or corner array to a voxel array,
  reducing each voxel's boundary. A merged face reads in piecewise, one piece
  per voxel it covers, and a piece carries what its face holds: one entry from
  a face array and four from a corner array.
- `swatchAvg` and its siblings take a voxel, face, or corner array to a swatch
  array, reducing each swatch's entries; a face or corner array reads in
  through the same voxel pieces, so a face spanning two swatches counts toward
  both.
- The unary `min`, `max`, `sum`, and `avg` take any array to plain across its
  whole domain.

`min` and `max` compose exactly across the rungs, so `swatchMin(voxelMin(e))`
is `swatchMin(e)`. The means and sums weigh their own rung:
`swatchAvg(voxelAvg(e))` weighs voxels evenly where `swatchAvg(e)` weighs
entries. `Min`, `Max`, and `Sum` keep the operand type; `Avg` returns `f32`.

A destination entry can be empty: a voxel with no faces, or a swatch whose
voxels have none. The avg, min, and max reductions error on an empty
destination entry; the sums answer `0`. A computation that has to survive an
empty entry is built from the sums:

```
aoFace = faceAvg(occlusion);
faceCount = swatchSum(face(1u32));
ao = swatchSum(aoFace) / f32(max(faceCount, 1));
```

## Backtick quoting and reserved names

Backticks quote what a bare identifier cannot hold: spaces, a leading digit, or
a reserved name. `foo bar` lexes as two names; the value is written
`` `foo bar` ``. Double quotes make string literals, never names.

The function names above and the literals `true` and `false` are reserved. A
name sharing one is reached by backtick-quoting: `` `min` `` is the name,
`min(...)` the function.

## Lexing

Whitespace separates tokens and is otherwise insignificant, with two
exceptions: inside a backtick-quoted name or a string literal it is literal, and
in a postfix it is forbidden. Numbers use maximal munch: `1.5` is one token, a
type suffix munches with its digits, `2u8` one token, and `.5` where an
expression is expected is a number. A suffix belongs to a whole number, so
`1.5u8` errors. In `2.rr` the munch stops at `2` because `2.` followed by a
letter is not a number; the attached dot then begins a swizzle, giving a vec2
splat. `2.5.rr` works the same way. The multi-character operators are single
tokens under the same munch, so a binding's `=` is never carved out of `==`.
