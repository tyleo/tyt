import * as check from "./check.ts";
import { Font } from "./font.ts";
import { Shape2d } from "./shape2d.ts";
import type { Vec2 } from "./types.ts";

/** The disk of `radius` around `center`. */
export function circle(center: Vec2, radius: number): Shape2d {
  return new Shape2d({
    kind: "circle",
    center: check.vec2("circle center", center),
    radius: check.finite("circle radius", radius),
  });
}

/** The rectangle between `min` and `max`, with corners cut or rounded. */
export function rect(
  min: Vec2,
  max: Vec2,
  options?: { chamfer?: number; round?: number },
): Shape2d {
  const option = check.knownOptions("rect", options, ["chamfer", "round"]);
  return new Shape2d({
    kind: "rect",
    min: check.vec2("rect min", min),
    max: check.vec2("rect max", max),
    chamfer: check.optional("rect chamfer", option.chamfer, check.finite),
    round: check.optional("rect round", option.round, check.finite),
  });
}

/** The ellipse with `radii` along u and v around `center`. */
export function ellipse(center: Vec2, radii: Vec2): Shape2d {
  return new Shape2d({
    kind: "ellipse",
    center: check.vec2("ellipse center", center),
    radii: check.vec2("ellipse radii", radii),
  });
}

/** The regular polygon with `sides` sides, each `radius` from `center`. */
export function ngon(center: Vec2, sides: number, radius: number): Shape2d {
  return new Shape2d({
    kind: "ngon",
    center: check.vec2("ngon center", center),
    sides: check.finite("ngon sides", sides),
    radius: check.finite("ngon radius", radius),
  });
}

/** The star with `points` tips between `innerRadius` and `outerRadius`. */
export function star(
  center: Vec2,
  points: number,
  outerRadius: number,
  innerRadius: number,
): Shape2d {
  return new Shape2d({
    kind: "star",
    center: check.vec2("star center", center),
    points: check.finite("star points", points),
    outerRadius: check.finite("star outerRadius", outerRadius),
    innerRadius: check.finite("star innerRadius", innerRadius),
  });
}

/** The outline through `points`, closed back to the first point. */
export function polygon(points: Vec2[]): Shape2d {
  return new Shape2d({
    kind: "polygon",
    points: check.list(
      "polygon points",
      points,
      "an array of Vec2s",
      check.vec2,
    ),
  });
}

/** The line through `points`, stroked `width` across. */
export function polyline(points: Vec2[], width: number): Shape2d {
  return new Shape2d({
    kind: "polyline",
    points: check.list(
      "polyline points",
      points,
      "an array of Vec2s",
      check.vec2,
    ),
    width: check.finite("polyline width", width),
  });
}

/**
 * The smooth curve through `points`, closed into an outline, or open and
 * stroked `width` across.
 */
export function spline(points: Vec2[], width?: number): Shape2d {
  return new Shape2d({
    kind: "spline",
    points: check.list(
      "spline points",
      points,
      "an array of Vec2s",
      check.vec2,
    ),
    width: check.optional("spline width", width, check.finite),
  });
}

/** The arc of `radius` between two angles, stroked `width` across. */
export function arc(
  center: Vec2,
  radius: number,
  fromDegrees: number,
  toDegrees: number,
  width: number,
  options?: { caps?: "flat" | "round" },
): Shape2d {
  const option = check.knownOptions("arc", options, ["caps"]);
  return new Shape2d({
    kind: "arc",
    center: check.vec2("arc center", center),
    radius: check.finite("arc radius", radius),
    fromDegrees: check.finite("arc fromDegrees", fromDegrees),
    toDegrees: check.finite("arc toDegrees", toDegrees),
    width: check.finite("arc width", width),
    caps: check.optional("arc caps", option.caps, check.caps),
  });
}

/** The pie slice of a disk of `radius` between two angles. */
export function sector(
  center: Vec2,
  radius: number,
  fromDegrees: number,
  toDegrees: number,
): Shape2d {
  return new Shape2d({
    kind: "sector",
    center: check.vec2("sector center", center),
    radius: check.finite("sector radius", radius),
    fromDegrees: check.finite("sector fromDegrees", fromDegrees),
    toDegrees: check.finite("sector toDegrees", toDegrees),
  });
}

/** The pointed lens `width` across from `a` to `b`. */
export function vesica(a: Vec2, b: Vec2, width: number): Shape2d {
  return new Shape2d({
    kind: "vesica",
    a: check.vec2("vesica a", a),
    b: check.vec2("vesica b", b),
    width: check.finite("vesica width", width),
  });
}

/** The rectangle between `min` and `max` topped with a half circle. */
export function arch(min: Vec2, max: Vec2): Shape2d {
  return new Shape2d({
    kind: "arch",
    min: check.vec2("arch min", min),
    max: check.vec2("arch max", max),
  });
}

/**
 * The line of `string` in `font` with capitals `size` tall, running along +u
 * with its baseline on v = 0. `align` puts the line's start, center, or end at
 * u = 0.
 */
export function text(
  font: Font,
  string: string,
  size: number,
  options?: { align?: "start" | "center" | "end" },
): Shape2d {
  const option = check.knownOptions("text", options, ["align"]);
  return new Shape2d({
    kind: "text",
    font: check.instance("text font", font, Font),
    string: check.text("text string", string),
    size: check.finite("text size", size),
    align: check.optional("text align", option.align, check.textAlign),
  });
}
