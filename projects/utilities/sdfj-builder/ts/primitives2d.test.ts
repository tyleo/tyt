import assert from "node:assert/strict";
import type { Entry } from "./entry.ts";
import { Font } from "./font.ts";
import {
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
  text,
  vesica,
} from "./primitives2d.ts";
import type { Shape2d } from "./shape2d.ts";
import { shapes2d } from "./test/tables.ts";

const SIGN = new Font({ kind: "installed", name: "sign", font: "Futura" });

const CASES: [string, () => Shape2d, Entry][] = [
  ["arc", () => arc([0, 0], 1, 0, 90, 0.1, { caps: "flat" }), {
    kind: "arc",
    center: [0, 0],
    radius: 1,
    fromDegrees: 0,
    toDegrees: 90,
    width: 0.1,
    caps: "flat",
  }],
  ["arch", () => arch([0, 0], [1, 2]), {
    kind: "arch",
    min: [0, 0],
    max: [1, 2],
  }],
  ["circle", () => circle([1, 2], 0.5), {
    kind: "circle",
    center: [1, 2],
    radius: 0.5,
  }],
  ["ellipse", () => ellipse([0, 0], [1, 2]), {
    kind: "ellipse",
    center: [0, 0],
    radii: [1, 2],
  }],
  ["ngon", () => ngon([0, 0], 6, 1), {
    kind: "ngon",
    center: [0, 0],
    sides: 6,
    radius: 1,
  }],
  ["polygon", () => polygon([[0, 0], [1, 0], [0, 1]]), {
    kind: "polygon",
    points: [[0, 0], [1, 0], [0, 1]],
  }],
  ["polyline", () => polyline([[0, 0], [1, 0]], 0.1), {
    kind: "polyline",
    points: [[0, 0], [1, 0]],
    width: 0.1,
  }],
  ["rect", () => rect([0, 0], [1, 2], { chamfer: 0.1 }), {
    kind: "rect",
    min: [0, 0],
    max: [1, 2],
    chamfer: 0.1,
  }],
  ["sector", () => sector([0, 0], 1, 0, 90), {
    kind: "sector",
    center: [0, 0],
    radius: 1,
    fromDegrees: 0,
    toDegrees: 90,
  }],
  ["spline outline", () => spline([[0, 0], [1, 0], [0, 1]]), {
    kind: "spline",
    points: [[0, 0], [1, 0], [0, 1]],
  }],
  ["spline stroke", () => spline([[0, 0], [1, 0]], 0.1), {
    kind: "spline",
    points: [[0, 0], [1, 0]],
    width: 0.1,
  }],
  ["star", () => star([0, 0], 5, 1, 0.5), {
    kind: "star",
    center: [0, 0],
    points: 5,
    outerRadius: 1,
    innerRadius: 0.5,
  }],
  ["text", () => text(SIGN, "OPEN", 0.1, { align: "center" }), {
    kind: "text",
    font: 0,
    string: "OPEN",
    size: 0.1,
    align: "center",
  }],
  ["vesica", () => vesica([0, 0], [0, 1], 0.25), {
    kind: "vesica",
    a: [0, 0],
    b: [0, 1],
    width: 0.25,
  }],
];

for (const [call, shape, entry] of CASES) {
  Deno.test(`${call} writes its arguments`, () => {
    assert.deepStrictEqual(shapes2d(shape()), [entry]);
  });
}

Deno.test("a list of points errors on a point the document cannot hold", () => {
  assert.throws(() => polygon([[0, 0], [1, Infinity]]), {
    message:
      "polygon points[1] must be a Vec2 of 2 finite numbers, not [1, Infinity]",
  });
  // deno-lint-ignore no-sparse-arrays
  assert.throws(() => polyline([[0, 0], , [1, 1]] as never, 0.1), {
    message:
      "polyline points[1] must be a Vec2 of 2 finite numbers, not undefined",
  });
});

Deno.test("a text takes a font from fonts", () => {
  assert.throws(() => text("Futura" as never, "OPEN", 0.1), {
    message: 'text font must be a Font, not "Futura"',
  });
});
