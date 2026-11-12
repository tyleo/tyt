// A PBR test chart, sized for --voxel-size 0.006. Five columns step the
// roughness. From the front: flat metal slabs, then gold, steel, and red
// plastic spheres, then a striped wall to reflect.
const v = 0.006;
const pitch = 28 * v;
const radius = 10 * v;
const roughnesses = [0.05, 0.2, 0.35, 0.5, 0.8];

const pale = material({ baseColor: "#E6E2D8", roughness: 0.7 });
const tile = material({ baseColor: "#A9B4BE", roughness: 0.7 });
const flat = (baseColor: string) => material({ baseColor, roughness: 0.8 });
const stripes = [flat("#D42A2A"), flat("#2FA84A"), flat("#2A55D4"), flat("#F2F2F2")];
const accents = [flat("#18C8D8"), flat("#E03AC0"), flat("#F2D21C"), flat("#35C24A")];
const postRed = flat("#D42A2A");
const postWhite = flat("#F2F2F2");

const steps: Step[] = [
  add(
    "floor",
    box([-0.48, -2 * v, -0.3], [0.48, 0, 0.444]),
    checker([pale, tile], 8 * v),
  ),
];

stripes.forEach((stripe, index) => {
  const x = -0.48 + index * 0.24;

  steps.push(add(`stripe ${index}`, box([x, 0, -0.3], [x + 0.24, 40 * v, -0.3 + 4 * v]), stripe));
});

roughnesses.forEach((roughness, column) => {
  const x = (column - 2) * pitch;
  const gold = material({ baseColor: "#FFC356", metallic: 1, roughness });
  const steel = material({ baseColor: "#B8B8B8", metallic: 1, roughness });
  const plastic = material({ baseColor: "#C02020", roughness });

  steps.push(
    add(`slab ${column}`, box([x - 11 * v, 0, 0.27], [x + 11 * v, v, 0.414]), steel),
    add(`gold ${column}`, sphere([x, radius, pitch], radius), gold),
    add(`steel ${column}`, sphere([x, radius, 0], radius), steel),
    add(`plastic ${column}`, sphere([x, radius, -pitch], radius), plastic),
    add(
      `post ${column}`,
      box([x - 7 * v, v, 0.3], [x - 5 * v, 16 * v, 0.3 + 2 * v]),
      bands([postRed, postWhite], 3 * v, { axis: "y" }),
    ),
    add(
      `block ${column}`,
      box([x + 2 * v, v, 0.312], [x + 6 * v, 5 * v, 0.336]),
      accents[column % accents.length],
    ),
  );
});

[-1.5, -0.5, 0.5, 1.5].forEach((between, index) => {
  const x = between * pitch;

  [0.5, -0.5].forEach((row, depth) => {
    const z = row * pitch;

    steps.push(
      add(
        `cube ${index} ${depth}`,
        box([x - 2 * v, 0, z - 2 * v], [x + 2 * v, 4 * v, z + 2 * v]),
        accents[(index + depth) % accents.length],
      ),
    );
  });
});

export default steps;
