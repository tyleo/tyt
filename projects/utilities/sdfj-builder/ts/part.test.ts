import assert from "node:assert/strict";
import { part } from "./part.ts";
import { add } from "./step.ts";
import { ball, document, white } from "./test/tables.ts";

Deno.test("part writes an object for its steps and a node", () => {
  const arm = part("arm", { pivot: [1, 0, 0], offset: [0, 1, 0] }, [
    add("arm", ball, white),
  ]);
  const { objects, nodes, rootNodes } = document([arm]);
  assert.deepStrictEqual(objects, [{ name: "arm", steps: [0] }]);
  assert.deepStrictEqual(nodes, [
    {
      name: "arm",
      pivot: [1, 0, 0],
      offset: [0, 1, 0],
      childObjects: [0],
      childNodes: [],
    },
    { name: "model", childObjects: [], childNodes: [0] },
  ]);
  assert.deepStrictEqual(rootNodes, [1]);
});

Deno.test("a part's rotations write in order", () => {
  const lid = part("lid", {
    pivot: [0, 1, 0],
    rotation: [["y", 120], ["x", -35]],
  }, [
    add("lid", ball, white),
  ]);
  const { nodes } = document([lid]);
  assert.deepStrictEqual(nodes[0], {
    name: "lid",
    pivot: [0, 1, 0],
    rotation: [{ axis: "y", degrees: 120 }, { axis: "x", degrees: -35 }],
    childObjects: [0],
    childNodes: [],
  });
});

Deno.test("an axis rotation takes an axis and finite degrees", () => {
  assert.throws(() => part("lid", { rotation: [["w", 30]] } as never, []), {
    message: 'part "lid" rotation[0][0] must be one of "x", "y", "z", not "w"',
  });
  assert.throws(() => part("lid", { rotation: [["x", Infinity]] }, []), {
    message: 'part "lid" rotation[0][1] must be a finite number, not Infinity',
  });
  assert.throws(() => part("lid", { rotation: ["x", 30] } as never, []), {
    message:
      'part "lid" rotation[0] must be an AxisRotation of an axis and finite degrees, not "x"',
  });
});

Deno.test("a joint writes only when it holds true", () => {
  const { nodes } = document([
    part("lid", { joint: true }, [add("lid", ball, white)]),
    part("base", { joint: false }, [add("base", ball, white)]),
  ]);
  assert.deepStrictEqual(nodes.slice(0, 2), [
    { name: "lid", joint: true, childObjects: [0], childNodes: [] },
    { name: "base", childObjects: [1], childNodes: [] },
  ]);
  assert.throws(() => part("lid", { joint: "yes" } as never, []), {
    message: 'part "lid" joint must be true or false, not "yes"',
  });
});

Deno.test("a step in several lists writes one entry", () => {
  const shared = add("shared", ball, white);
  const { steps, objects } = document([
    part("left", {}, [shared]),
    part("right", {}, [shared]),
  ]);
  assert.deepStrictEqual(steps.length, 1);
  assert.deepStrictEqual(objects, [
    { name: "left", steps: [0] },
    { name: "right", steps: [0] },
  ]);
});

Deno.test("a part's list holds only steps and parts", () => {
  assert.throws(() => part("arm", {}, [ball as never]), {
    message: 'part "arm" list[0] must be a Step or a Part, not a Shape3d',
  });
  assert.throws(() => part("arm", { pivot: [0, 0, 0], turn: 1 } as never, []), {
    message: 'part "arm" takes no option "turn"',
  });
});
