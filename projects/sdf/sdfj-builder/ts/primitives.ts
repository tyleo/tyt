import * as check from "./check.ts";
import { Shape3d } from "./shape3d.ts";
import type { Axis, Side, Vec3 } from "./types.ts";

/** The box between corners `min` and `max`, with edges rounded by `round`. */
export function box(
  min: Vec3,
  max: Vec3,
  options?: { round?: number },
): Shape3d {
  const option = check.knownOptions("box", options, ["round"]);
  return new Shape3d({
    kind: "box",
    min: check.vec3("box min", min),
    max: check.vec3("box max", max),
    round: check.optional("box round", option.round, check.finite),
  });
}

/** The twelve edges of the box between `min` and `max` as square bars. */
export function boxFrame(min: Vec3, max: Vec3, thickness: number): Shape3d {
  return new Shape3d({
    kind: "boxFrame",
    min: check.vec3("boxFrame min", min),
    max: check.vec3("boxFrame max", max),
    thickness: check.finite("boxFrame thickness", thickness),
  });
}

/** The sphere of `radius` around `center`. */
export function sphere(center: Vec3, radius: number): Shape3d {
  return new Shape3d({
    kind: "sphere",
    center: check.vec3("sphere center", center),
    radius: check.finite("sphere radius", radius),
  });
}

/** The ellipsoid with `radii` along the axes around `center`. */
export function ellipsoid(center: Vec3, radii: Vec3): Shape3d {
  return new Shape3d({
    kind: "ellipsoid",
    center: check.vec3("ellipsoid center", center),
    radii: check.vec3("ellipsoid radii", radii),
  });
}

/** The cylinder of `radius` from `a` to `b`, with rims rounded by `round`. */
export function cylinder(
  a: Vec3,
  b: Vec3,
  radius: number,
  options?: { round?: number },
): Shape3d {
  const option = check.knownOptions("cylinder", options, ["round"]);
  return new Shape3d({
    kind: "cylinder",
    a: check.vec3("cylinder a", a),
    b: check.vec3("cylinder b", b),
    radius: check.finite("cylinder radius", radius),
    round: check.optional("cylinder round", option.round, check.finite),
  });
}

/** The cone from `radiusA` at `a` to `radiusB` at `b`. */
export function cone(
  a: Vec3,
  b: Vec3,
  radiusA: number,
  radiusB: number,
): Shape3d {
  return new Shape3d({
    kind: "cone",
    a: check.vec3("cone a", a),
    b: check.vec3("cone b", b),
    radiusA: check.finite("cone radiusA", radiusA),
    radiusB: check.finite("cone radiusB", radiusB),
  });
}

/** The points within `radius` of the segment from `a` to `b`. */
export function capsule(a: Vec3, b: Vec3, radius: number): Shape3d {
  return new Shape3d({
    kind: "capsule",
    a: check.vec3("capsule a", a),
    b: check.vec3("capsule b", b),
    radius: check.finite("capsule radius", radius),
  });
}

/** The spheres at `a` and `b` and the taper between them. */
export function roundCone(
  a: Vec3,
  b: Vec3,
  radiusA: number,
  radiusB: number,
): Shape3d {
  return new Shape3d({
    kind: "roundCone",
    a: check.vec3("roundCone a", a),
    b: check.vec3("roundCone b", b),
    radiusA: check.finite("roundCone radiusA", radiusA),
    radiusB: check.finite("roundCone radiusB", radiusB),
  });
}

/** The ring around `axis` through `center`, cut to the arc `from` to `to`. */
export function torus(
  center: Vec3,
  ringRadius: number,
  tubeRadius: number,
  options?: { axis?: Axis; from?: number; to?: number },
): Shape3d {
  const option = check.knownOptions("torus", options, ["axis", "from", "to"]);
  return new Shape3d({
    kind: "torus",
    center: check.vec3("torus center", center),
    ringRadius: check.finite("torus ringRadius", ringRadius),
    tubeRadius: check.finite("torus tubeRadius", tubeRadius),
    axis: check.optional("torus axis", option.axis, check.axis),
    from: check.optional("torus from", option.from, check.finite),
    to: check.optional("torus to", option.to, check.finite),
  });
}

/** The octahedron reaching `radius` along each axis from `center`. */
export function octahedron(center: Vec3, radius: number): Shape3d {
  return new Shape3d({
    kind: "octahedron",
    center: check.vec3("octahedron center", center),
    radius: check.finite("octahedron radius", radius),
  });
}

/** The pyramid on a square base `width` across with its apex `height` up. */
export function pyramid(
  baseCenter: Vec3,
  width: number,
  height: number,
): Shape3d {
  return new Shape3d({
    kind: "pyramid",
    baseCenter: check.vec3("pyramid baseCenter", baseCenter),
    width: check.finite("pyramid width", width),
    height: check.finite("pyramid height", height),
  });
}

/** Everything past `at` toward `side`. */
export function halfSpace(side: Side, at: number): Shape3d {
  return new Shape3d({
    kind: "halfSpace",
    side: check.side("halfSpace side", side),
    at: check.finite("halfSpace at", at),
  });
}
