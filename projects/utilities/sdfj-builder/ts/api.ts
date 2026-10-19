// The names a model file can use without importing them.
export {
  intersect,
  smoothIntersect,
  smoothSubtract,
  smoothUnion,
  subtract,
  union,
} from "./booleans.ts";
export {
  int,
  type IntValue,
  json,
  type JsonValue,
  type Material,
  material,
  type Properties,
  type Value,
} from "./material.ts";
export { type Part, part } from "./part.ts";
export {
  bands,
  cells,
  checker,
  gradient,
  grain,
  noise,
  type Pattern,
  speckle,
} from "./pattern.ts";
export {
  box,
  boxFrame,
  capsule,
  cone,
  cylinder,
  ellipsoid,
  halfSpace,
  octahedron,
  pyramid,
  roundCone,
  sphere,
  torus,
} from "./primitives.ts";
export {
  arc,
  arch,
  circle,
  ellipse,
  ngon,
  polygon,
  polyline,
  rect,
  sector,
  spline,
  star,
  vesica,
} from "./primitives2d.ts";
export { extrude, lathe, revolve } from "./profiles.ts";
export { shades } from "./shades.ts";
export { type Shape2d } from "./shape2d.ts";
export { type Shape3d } from "./shape3d.ts";
export {
  add,
  carve,
  coat,
  type CoatOptions,
  paint,
  type PaintOptions,
  set,
  type Step,
} from "./step.ts";
export type { Axes, Axis, AxisRotation, Side, Vec2, Vec3 } from "./types.ts";
