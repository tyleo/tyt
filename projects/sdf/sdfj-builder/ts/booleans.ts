import * as check from "./check.ts";
import { Shape2d } from "./shape2d.ts";
import { Shape3d } from "./shape3d.ts";

/** The region inside any of `shapes`. */
export function union<S extends Shape3d | Shape2d>(...shapes: S[]): S {
  const first = nonEmpty("union", shapes)[0];
  return combine("union", "shapes[0]", first, (path, shape) => ({
    kind: "union",
    shapes: sameShapes(`${path} shapes`, shapes, shape),
  }));
}

/** The region inside all of `shapes`. */
export function intersect<S extends Shape3d | Shape2d>(...shapes: S[]): S {
  const first = nonEmpty("intersect", shapes)[0];
  return combine("intersect", "shapes[0]", first, (path, shape) => ({
    kind: "intersect",
    shapes: sameShapes(`${path} shapes`, shapes, shape),
  }));
}

/** The region inside `base` and outside every one of `cutters`. */
export function subtract<S extends Shape3d | Shape2d>(
  base: S,
  ...cutters: S[]
): S {
  return combine("subtract", "base", base, (path, shape) => ({
    kind: "subtract",
    base,
    cutters: sameShapes(`${path} cutters`, cutters, shape),
  }));
}

/** The union of `shapes` with fillets reaching `radius` where they meet. */
export function smoothUnion<S extends Shape3d | Shape2d>(
  radius: number,
  ...shapes: S[]
): S {
  const first = nonEmpty("smoothUnion", shapes)[0];
  return combine("smoothUnion", "shapes[0]", first, (path, shape) => ({
    kind: "smoothUnion",
    radius: check.finite(`${path} radius`, radius),
    shapes: sameShapes(`${path} shapes`, shapes, shape),
  }));
}

/** The intersection of `shapes` with edges rounded over `radius`. */
export function smoothIntersect<S extends Shape3d | Shape2d>(
  radius: number,
  ...shapes: S[]
): S {
  const first = nonEmpty("smoothIntersect", shapes)[0];
  return combine("smoothIntersect", "shapes[0]", first, (path, shape) => ({
    kind: "smoothIntersect",
    radius: check.finite(`${path} radius`, radius),
    shapes: sameShapes(`${path} shapes`, shapes, shape),
  }));
}

/** `base` minus `cutters` with edges rounded over `radius`. */
export function smoothSubtract<S extends Shape3d | Shape2d>(
  radius: number,
  base: S,
  ...cutters: S[]
): S {
  return combine("smoothSubtract", "base", base, (path, shape) => ({
    kind: "smoothSubtract",
    radius: check.finite(`${path} radius`, radius),
    base,
    cutters: sameShapes(`${path} cutters`, cutters, shape),
  }));
}

/** The class of a boolean's shapes. */
type ShapeClass = abstract new (...args: never[]) => Shape3d | Shape2d;

/**
 * The boolean `call` makes in the table of `first`, the shape at `firstKey`.
 * `fields` builds the entry from the class every other shape has to share.
 */
function combine<S extends Shape3d | Shape2d>(
  call: string,
  firstKey: string,
  first: S | undefined,
  fields: (
    path: string,
    shape: ShapeClass,
  ) => Readonly<Record<string, unknown>>,
): S {
  if (first instanceof Shape3d) {
    return new Shape3d(fields(call, Shape3d)) as S;
  }
  if (first instanceof Shape2d) {
    return new Shape2d(fields(call, Shape2d)) as S;
  }
  return check.fail(`${call} ${firstKey}`, "a Shape3d or a Shape2d", first);
}

/** `shapes`, which errors when it holds no shape. */
function nonEmpty<S>(call: string, shapes: S[]): S[] {
  return shapes.length > 0
    ? shapes
    : check.fail(`${call} shapes`, "at least one shape", shapes);
}

/** A copy of `shapes`, each an instance of `shape`. */
function sameShapes(
  path: string,
  shapes: readonly unknown[],
  shape: ShapeClass,
): (Shape3d | Shape2d)[] {
  return shapes.map((item, index) =>
    check.instance(`${path}[${index}]`, item, shape)
  );
}
