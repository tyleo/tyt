const tree = part("tree", {}, [
  add("trunk", cylinder([0, 0, 0], [0, 0.5, 0], 0.05), mat.bark),
  add("crown", sphere([0, 0.65, 0], 0.25), mat.leaf),
]);

const fur = material({ baseColor: "#C8B8A0" });
const rabbit = part("rabbit", {}, [
  add("body", ellipsoid([0, 0.05, 0], [0.05, 0.05, 0.075]), fur),
]);

export default [
  part("tree.1", { offset: [-0.75, 0, 0] }, [
    tree,
    part("rabbit.1", { offset: [0.15, 0, 0.15] }, [rabbit]),
  ]),
  part("tree.2", { offset: [0, 0, -0.5] }, [tree]),
  part("tree.3", { offset: [0.75, 0, 0] }, [
    tree,
    part("rabbit.2", { offset: [-0.15, 0, 0.15] }, [rabbit]),
  ]),
];
