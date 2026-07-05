# Implementation decisions

_Part of the [ty-math Adoption Plan](../README.md)._

Code-level decisions made while executing the [checklist](../checklist.md),
recorded as they land. The plan-level decisions and their rationale live in the
[README](README.md#decisions); this log is for the finer choices a reviewer of the
Rust would want explained, for example the confirmed name and signature of each new
`ty-math` method, whether a borderline vmax adoption was taken, and what the Track C
investigation found in each `internal/` file.

No work has landed yet. Add a section under the relevant track as its first chunk
lands.

## Track A: harmless cleanups

_Pending._

## Track B: ty-math additions and adoption

_Pending. Record the final method names against the Q3 recommendations and any
signature that differed from the plan (for example whether the vector cast landed
as `to_i32`, `as_i32`, or a combined `round_to_i32`)._

## Track C: heavier logic under internal/

_Pending. Record the catalog of the three patterns per file, which adoptions were
safe, and each larger primitive filed as a new item with its target type._
