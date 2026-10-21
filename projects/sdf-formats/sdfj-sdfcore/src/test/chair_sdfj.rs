/// The legs and seat of a chair.
pub const CHAIR_SDFJ: &str = r##"{
  "version": 1,
  "shapes3d": [
    { "kind": "lathe", "points": [[0.0375, 0], [0.03, 0.15], [0.045, 0.225], [0.03, 0.425]] },
    { "kind": "translate", "shape": 0, "offset": [0.2, 0, 0.2] },
    { "kind": "mirror", "shape": 1, "axes": "xz" },
    { "kind": "box", "min": [-0.25, 0.425, -0.25], "max": [0.25, 0.475, 0.25] }
  ],
  "shapes2d": [],
  "materials": [
    { "kind": "material", "properties": { "baseColor": "#8A5A2B", "roughness": 0.8 } },
    { "kind": "shade", "shades": 0, "index": 0 },
    { "kind": "shade", "shades": 0, "index": 1 },
    { "kind": "shade", "shades": 0, "index": 2 }
  ],
  "shades": [{ "base": 0, "count": 3 }],
  "patterns": [
    { "kind": "grain", "materials": [1, 2, 3], "axis": "y", "period": 0.05, "seed": 1 },
    { "kind": "grain", "materials": [1, 2, 3], "axis": "x", "period": 0.05, "seed": 2 }
  ],
  "steps": [
    { "kind": "add", "name": "legs", "shape": 2, "pattern": 0 },
    { "kind": "add", "name": "seat", "shape": 3, "pattern": 1 }
  ],
  "objects": [{ "name": "chair", "steps": [0, 1] }],
  "nodes": [{ "name": "chair", "childObjects": [0], "childNodes": [] }],
  "rootNodes": [0]
}
"##;
