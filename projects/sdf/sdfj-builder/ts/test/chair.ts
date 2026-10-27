const oak = (axis: Axis, seed: number) => grain(mat.oak, 0.05, { axis, seed });
const leg = lathe([
  [0.0375, 0],
  [0.03, 0.15],
  [0.045, 0.225],
  [0.03, 0.425],
]).translate([0.2, 0, 0.2]);

export default [
  add("legs", leg.mirror("xz"), oak("y", 1)),
  add("seat", box([-0.25, 0.425, -0.25], [0.25, 0.475, 0.25]), oak("x", 2)),
];
