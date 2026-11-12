// A glass test chart, sized for --voxel-size 0.006. From the back: panes
// that throw tinted shadows, then a tank of water, a block of solid glass,
// and a lamp in a glass case, then a wall whose shadow falls across a
// glass slab.
const v = 0.006;

const pale = material({ baseColor: "#E6E2D8", roughness: 0.7 });
const tile = material({ baseColor: "#A9B4BE", roughness: 0.7 });
const flat = (baseColor: string) => material({ baseColor, roughness: 0.8 });
const glass = (baseColor: string) => material({ baseColor, roughness: 0.05, transmission: 1 });
const frame = flat("#2A2A2E");

const panes = [
  glass("#FFFFFF"),
  glass("#FF3030"),
  glass("#30D050"),
  glass("#3060FF"),
  material({ baseColor: "#40404080", roughness: 0.3 }),
];

const steps: Step[] = [
  add("floor", box([-0.48, -2 * v, -0.3], [0.48, 0, 0.444]), checker([pale, tile], 8 * v)),
];

// The panes stand across the back, each on a dark sill.
panes.forEach((pane, index) => {
  const x = (index - 2) * 30 * v;

  steps.push(
    add(`sill ${index}`, box([x - 11 * v, 0, -0.2], [x + 11 * v, 2 * v, -0.2 + 2 * v]), frame),
    add(`pane ${index}`, box([x - 10 * v, 2 * v, -0.2], [x + 10 * v, 34 * v, -0.2 + 2 * v]), pane),
  );
});

// A tank of water with gravel, a cube, and a ball in it.
const tank = { x: -0.27, z: 0.06, half: [22 * v, 14 * v], height: 24 * v };
const wall = glass("#E8F4FF");
const water = material({ baseColor: "#58B8E8", roughness: 0.05, transmission: 0.9 });
const gravel = [flat("#C8A878"), flat("#A88858"), flat("#E0C8A0")];

steps.push(
  add(
    "tank",
    box(
      [tank.x - tank.half[0], 0, tank.z - tank.half[1]],
      [tank.x + tank.half[0], tank.height, tank.z + tank.half[1]],
    ),
    wall,
  ),
  add(
    "water",
    box(
      [tank.x - tank.half[0] + v, v, tank.z - tank.half[1] + v],
      [tank.x + tank.half[0] - v, tank.height - 4 * v, tank.z + tank.half[1] - v],
    ),
    water,
  ),
  carve(
    "air",
    box(
      [tank.x - tank.half[0] + v, tank.height - 4 * v, tank.z - tank.half[1] + v],
      [tank.x + tank.half[0] - v, tank.height, tank.z + tank.half[1] - v],
    ),
  ),
  add(
    "gravel",
    box(
      [tank.x - tank.half[0] + v, v, tank.z - tank.half[1] + v],
      [tank.x + tank.half[0] - v, 4 * v, tank.z + tank.half[1] - v],
    ),
    noise(gravel, 2 * v, { seed: 3 }),
  ),
  add("cube", box([tank.x - 14 * v, 4 * v, tank.z - 4 * v], [tank.x - 6 * v, 12 * v, tank.z + 4 * v]), flat("#D42A2A")),
  add("ball", sphere([tank.x + 8 * v, 10 * v, tank.z], 6 * v), flat("#F2D21C")),
);

// A block of solid glass with a gold ball in it.
const block = { x: 0, z: 0.06 };

steps.push(
  add(
    "block",
    box([block.x - 10 * v, 0, block.z - 10 * v], [block.x + 10 * v, 20 * v, block.z + 10 * v]),
    glass("#B8F0C8"),
  ),
  add(
    "gold",
    sphere([block.x, 10 * v, block.z], 5 * v),
    material({ baseColor: "#FFC356", metallic: 1, roughness: 0.3 }),
  ),
);

// A lamp in a glass case.
const lamp = { x: 0.27, z: 0.06 };

steps.push(
  add("base", box([lamp.x - 9 * v, 0, lamp.z - 9 * v], [lamp.x + 9 * v, 2 * v, lamp.z + 9 * v]), frame),
  add(
    "case",
    box([lamp.x - 8 * v, 2 * v, lamp.z - 8 * v], [lamp.x + 8 * v, 20 * v, lamp.z + 8 * v]),
    glass("#FFE8C0"),
  ),
  carve(
    "hollow",
    box([lamp.x - 7 * v, 2 * v, lamp.z - 7 * v], [lamp.x + 7 * v, 19 * v, lamp.z + 7 * v]),
  ),
  add(
    "lamp",
    sphere([lamp.x, 6 * v, lamp.z], 4 * v),
    material({ baseColor: "#FFD080", emissiveColor: "#FFB040", emissiveStrength: 4 }),
  ),
);

// A wall whose shadow falls across a slab of glass, beside cubes behind
// a pane.
steps.push(
  add("wall", box([-0.36, 0, 0.27], [-0.36 + 3 * v, 30 * v, 0.27 + 20 * v]), flat("#8A5A2B")),
  add("slab", box([-0.33, 0, 0.26], [-0.09, 3 * v, 0.41]), glass("#C0E0FF")),
  add("screen", box([0.06, 0, 0.36], [0.36, 22 * v, 0.36 + 2 * v]), glass("#FFB0D0")),
);

["#D42A2A", "#2FA84A", "#2A55D4", "#F2F2F2"].forEach((color, index) => {
  const x = 0.1 + index * 0.07;

  steps.push(add(`behind ${index}`, box([x, 0, 0.29], [x + 6 * v, 10 * v, 0.29 + 6 * v]), flat(color)));
});

export default steps;
