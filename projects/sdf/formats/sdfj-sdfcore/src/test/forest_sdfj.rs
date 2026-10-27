/// One tree in three places and one rabbit under two of the trees.
pub const FOREST_SDFJ: &str = r##"{
  "version": 1,
  "shapes3d": [
    { "kind": "cylinder", "a": [0, 0, 0], "b": [0, 0.5, 0], "radius": 0.05 },
    { "kind": "sphere", "center": [0, 0.65, 0], "radius": 0.25 },
    { "kind": "ellipsoid", "center": [0, 0.05, 0], "radii": [0.05, 0.05, 0.075] }
  ],
  "shapes2d": [],
  "materials": [
    { "kind": "material", "properties": { "baseColor": "#5A3A22", "roughness": 0.9 } },
    { "kind": "material", "properties": { "baseColor": "#3F7A2E", "roughness": 0.7 } },
    { "kind": "material", "properties": { "baseColor": "#C8B8A0" } }
  ],
  "shades": [],
  "patterns": [],
  "steps": [
    { "kind": "add", "name": "trunk", "shape": 0, "material": 0 },
    { "kind": "add", "name": "crown", "shape": 1, "material": 1 },
    { "kind": "add", "name": "body", "shape": 2, "material": 2 }
  ],
  "objects": [
    { "name": "tree", "steps": [0, 1] },
    { "name": "rabbit", "steps": [2] }
  ],
  "nodes": [
    { "name": "tree", "childObjects": [0], "childNodes": [] },
    { "name": "rabbit", "childObjects": [1], "childNodes": [] },
    { "name": "rabbit.1", "offset": [0.15, 0, 0.15], "childObjects": [], "childNodes": [1] },
    { "name": "tree.1", "offset": [-0.75, 0, 0], "childObjects": [], "childNodes": [0, 2] },
    { "name": "tree.2", "offset": [0, 0, -0.5], "childObjects": [], "childNodes": [0] },
    { "name": "rabbit.2", "offset": [-0.15, 0, 0.15], "childObjects": [], "childNodes": [1] },
    { "name": "tree.3", "offset": [0.75, 0, 0], "childObjects": [], "childNodes": [0, 5] },
    { "name": "forest", "childObjects": [], "childNodes": [3, 4, 6] }
  ],
  "rootNodes": [7]
}
"##;
