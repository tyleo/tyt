import assert from "node:assert/strict";
import { libraryGlobals } from "./library.ts";
import { material } from "./material.ts";
import { part } from "./part.ts";
import { grain } from "./pattern.ts";
import { box } from "./primitives.ts";
import { extrude } from "./profiles.ts";
import { type SdfjDocument, sdfjDocument } from "./sdfj_document.ts";
import { sdfjDocumentFromJson } from "./sdfj_json.ts";
import { add, coat } from "./step.ts";
import { ball, disk, library, white } from "./test/tables.ts";

const walnut = material({ baseColor: "#5C4033", roughness: 0.6 });

const leg = extrude(disk, { from: 0, to: 0.4 }).translate([0.15, 0, 0.15]);

const stool = part("stool", { pivot: [0, 0.2, 0] }, [
  add("legs", leg.mirror("xz"), grain(walnut, { axis: "y", seed: 1 })),
  coat("trim", walnut, 0.05, {
    sides: ["+y"],
    within: box([-1, 0, -1], [1, 1, 1]),
  }),
  part("cushion", { offset: [0, 0.4, 0], rotation: [["y", 45]] }, [
    add("cushion", ball, white),
  ]),
]);

/** The library of the stool's exports. */
const WOODS = library("woods", [
  ["leg", leg],
  ["stool", stool],
  ["walnut", walnut],
]);

Deno.test("a library part writes the document the part wrote", () => {
  const { lib } = libraryGlobals([WOODS]);
  assert.deepStrictEqual(
    sdfjDocument("woods", [lib.parts.stool]),
    sdfjDocument("woods", [stool], [
      ["leg", leg],
      ["stool", stool],
      ["walnut", walnut],
    ]),
  );
});

Deno.test("a library value reads once however often a model uses it", () => {
  const { lib, mat } = libraryGlobals([WOODS]);
  assert.equal(mat.walnut, lib.materials.walnut);
  assert.equal(lib.parts.stool, lib.parts.stool);
  const { materials, names } = sdfjDocument("model", [
    add("seat", ball, mat.walnut),
    add("back", ball, mat.walnut),
  ]);
  assert.deepStrictEqual(materials, [
    { kind: "material", properties: { baseColor: "#5C4033", roughness: 0.6 } },
  ]);
  assert.deepStrictEqual(names.materials, { walnut: 0 });
});

Deno.test("a later library wins a name", () => {
  const pale = library("pale", [["walnut", white]]);
  const { mat } = libraryGlobals([WOODS, pale]);
  const { materials } = sdfjDocument("model", [add("seat", ball, mat.walnut)]);
  assert.deepStrictEqual(materials, [
    { kind: "material", properties: { baseColor: "#FFFFFF" } },
  ]);
});

Deno.test("an entry takes every name its library gives it", () => {
  const { mat } = libraryGlobals([
    library("woods", [["walnut", walnut], ["wood", walnut]]),
  ]);
  const { names } = sdfjDocument("model", [add("seat", ball, mat.wood)]);
  assert.deepStrictEqual(names.materials, { walnut: 0, wood: 0 });
});

Deno.test("reading a name no library holds errors", () => {
  const { lib, mat } = libraryGlobals([WOODS, library("props", [])]);
  assert.throws(() => mat.oak, {
    message: 'mat has no material "oak" in the libraries "woods", "props"',
  });
  assert.throws(() => lib.shapes3d.stool, {
    message:
      'lib.shapes3d has no 3D shape "stool" in the libraries "woods", "props"',
  });
  assert.throws(() => Reflect.get(lib, "part"), {
    message: 'lib has no table "part"',
  });
  assert.throws(() => libraryGlobals([WOODS]).lib.steps.legs, {
    message: 'lib.steps has no step "legs" in the library "woods"',
  });
});

Deno.test("a node a part cannot write errors", () => {
  const nodes = (edit: (document: SdfjDocument) => void) => {
    const document = sdfjDocumentFromJson(structuredClone(WOODS.document));
    edit(document);
    return () => libraryGlobals([{ name: "woods", document }]).lib.parts.stool;
  };
  // The cushion writes nodes[1] and objects[0], and the stool writes nodes[2]
  // and objects[1].
  assert.throws(
    nodes((document) => {
      document.nodes[2] = {
        ...document.nodes[2],
        childObjects: [0, 1],
      };
    }),
    {
      message:
        'library "woods" nodes[2] holds 2 objects, and a part holds at most one',
    },
  );
  assert.throws(
    nodes((document) => {
      document.objects[1] = { ...document.objects[1], name: "seat" };
    }),
    {
      message:
        `library "woods" nodes[2] holds objects[1] named "seat", and a part's object takes the part's name "stool"`,
    },
  );
  assert.throws(
    nodes((document) => {
      document.objects[1] = { ...document.objects[1], steps: [] };
    }),
    {
      message:
        'library "woods" nodes[2] holds objects[1] with no steps, and a part writes an object only for its steps',
    },
  );
  assert.throws(
    nodes((document) => {
      document.nodes[1] = { ...document.nodes[1], childObjects: [1] };
    }),
    {
      message:
        'library "woods" nodes[1] shares objects[1] with nodes[2], and each part holds its own object',
    },
  );
});
