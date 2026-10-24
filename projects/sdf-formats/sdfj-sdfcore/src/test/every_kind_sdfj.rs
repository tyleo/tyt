/// One entry of every kind with every optional key, each referencing earlier
/// entries, and a name in every names table.
pub const EVERY_KIND_SDFJ: &str = r##"{
  "version": 1,
  "shapes3d": [
    { "kind": "box", "min": [0, 0, 0], "max": [1, 1, 1], "round": 0.1 },
    { "kind": "boxFrame", "min": [0, 0, 0], "max": [1, 1, 1], "thickness": 0.1 },
    { "kind": "sphere", "center": [0, 0, 0], "radius": 1 },
    { "kind": "ellipsoid", "center": [0, 0, 0], "radii": [1, 2, 3] },
    { "kind": "cylinder", "a": [0, 0, 0], "b": [0, 1, 0], "radius": 0.5, "round": 0.1 },
    { "kind": "cone", "a": [0, 0, 0], "b": [0, 1, 0], "radiusA": 0.5, "radiusB": 0 },
    { "kind": "roundCone", "a": [0, 0, 0], "b": [0, 1, 0], "radiusA": 0.5, "radiusB": 0.25 },
    { "kind": "capsule", "a": [0, 0, 0], "b": [0, 1, 0], "radius": 0.5 },
    { "kind": "torus", "center": [0, 0, 0], "ringRadius": 1, "tubeRadius": 0.25, "axis": "x", "from": 0, "to": 180 },
    { "kind": "octahedron", "center": [0, 0, 0], "radius": 1 },
    { "kind": "pyramid", "baseCenter": [0, 0, 0], "width": 1, "height": 2 },
    { "kind": "halfSpace", "side": "-z", "at": 0.5 },
    { "kind": "extrude", "profile": 0, "axis": "y", "from": 0, "to": 1 },
    { "kind": "revolve", "profile": 0, "axis": "z", "center": [0, 1, 0] },
    { "kind": "lathe", "points": [[0.5, 0], [0.25, 1]], "axis": "x", "center": [0, 1, 0] },
    { "kind": "union", "shapes": [0, 1] },
    { "kind": "intersect", "shapes": [0, 2] },
    { "kind": "subtract", "base": 0, "cutters": [1, 2] },
    { "kind": "smoothUnion", "radius": 0.1, "shapes": [0, 1] },
    { "kind": "smoothIntersect", "radius": 0.1, "shapes": [0, 2] },
    { "kind": "smoothSubtract", "radius": 0.1, "base": 0, "cutters": [1] },
    { "kind": "translate", "shape": 0, "offset": [1, 0, 0] },
    { "kind": "rotate", "shape": 0, "axis": "y", "degrees": 30, "pivot": [0, 1, 0] },
    { "kind": "orient", "shape": 0, "from": [0, 1, 0], "to": [1, 0, 0], "pivot": [0, 0, 1] },
    { "kind": "scale", "shape": 0, "factor": [1, 2, 3], "pivot": [1, 0, 0] },
    { "kind": "mirror", "shape": 0, "axes": "xyz", "center": [0, 0, 1] },
    { "kind": "repeat", "shape": 0, "step": [1, 0, 0], "count": [4, 1, 1] },
    { "kind": "repeatPolar", "shape": 0, "axis": "y", "count": 12, "center": [0, 0, 0] },
    { "kind": "offset", "shape": 0, "distance": -0.1 },
    { "kind": "shell", "shape": 0, "thickness": 0.1 },
    { "kind": "elongate", "shape": 0, "lengths": [1, 0, 0], "center": [0.5, 0.5, 0.5] },
    { "kind": "twist", "shape": 0, "axis": "y", "degreesPerMeter": 90, "center": [0, 0, 0] },
    { "kind": "bend", "shape": 0, "along": "x", "toward": "+y", "radius": 2, "pivot": [0, 0, 0] },
    { "kind": "displace", "shape": 0, "amplitude": 0.1, "scale": 0.5, "octaves": 3, "seed": 7 },
    { "kind": "sweep", "profile": 0, "points": [[0, 0, 0], [1, 1, 0]], "scales": [1, 0.5], "up": [0, 0, 1] }
  ],
  "shapes2d": [
    { "kind": "circle", "center": [0, 0], "radius": 1 },
    { "kind": "rect", "min": [0, 0], "max": [1, 2], "chamfer": 0.1, "round": 0.1 },
    { "kind": "ellipse", "center": [0, 0], "radii": [1, 2] },
    { "kind": "ngon", "center": [0, 0], "sides": 6, "radius": 1 },
    { "kind": "star", "center": [0, 0], "points": 5, "outerRadius": 1, "innerRadius": 0.5 },
    { "kind": "polygon", "points": [[0, 0], [1, 0], [0, 1]] },
    { "kind": "polyline", "points": [[0, 0], [1, 0]], "width": 0.1 },
    { "kind": "arc", "center": [0, 0], "radius": 1, "fromDegrees": 0, "toDegrees": 90, "width": 0.1, "caps": "round" },
    { "kind": "sector", "center": [0, 0], "radius": 1, "fromDegrees": 0, "toDegrees": 90 },
    { "kind": "vesica", "a": [0, 0], "b": [0, 1], "width": 0.25 },
    { "kind": "arch", "min": [0, 0], "max": [1, 2] },
    { "kind": "union", "shapes": [0, 1] },
    { "kind": "intersect", "shapes": [0, 2] },
    { "kind": "subtract", "base": 0, "cutters": [1] },
    { "kind": "smoothUnion", "radius": 0.1, "shapes": [0, 1] },
    { "kind": "smoothIntersect", "radius": 0.1, "shapes": [0, 2] },
    { "kind": "smoothSubtract", "radius": 0.1, "base": 0, "cutters": [1, 2] },
    { "kind": "translate", "shape": 0, "offset": [1, 0] },
    { "kind": "rotate", "shape": 0, "degrees": 45, "pivot": [1, 0] },
    { "kind": "scale", "shape": 0, "factor": [2, 1], "pivot": [0, 1] },
    { "kind": "mirror", "shape": 0, "axes": "uv", "center": [1, 1] },
    { "kind": "repeat", "shape": 0, "step": [1, 1], "count": [2, 3] },
    { "kind": "repeatPolar", "shape": 0, "count": 6, "center": [0, 1] },
    { "kind": "offset", "shape": 0, "distance": 0.1 },
    { "kind": "shell", "shape": 0, "thickness": 0.1 },
    { "kind": "spline", "points": [[0, 0], [1, 0], [0, 1]] },
    { "kind": "spline", "points": [[0, 0], [1, 0]], "width": 0.1 }
  ],
  "materials": [
    {
      "kind": "material",
      "properties": {
        "baseColor": "#3A3F44",
        "locked": true,
        "lootTier": { "kind": "int", "value": 3 },
        "slots": { "kind": "int", "value": [1, 2] },
        "tags": { "kind": "json", "value": { "a": [1.5, null, true, "s"] } },
        "roughness": 0.25,
        "tint": [0.5, 0.25, 1]
      }
    },
    { "kind": "material", "properties": { "baseColor": "#D8F0FF" } },
    { "kind": "shade", "shades": 0, "index": 0 },
    { "kind": "shade", "shades": 0, "index": 1 }
  ],
  "shades": [{ "base": 1, "count": 2, "spread": 0.1 }],
  "patterns": [
    { "kind": "bands", "materials": [0, 1], "axis": "x", "period": 0.1, "warp": 0.05, "seed": 3 },
    { "kind": "grain", "materials": [2, 3], "axis": "y", "period": 0.1, "warp": 0.05, "seed": 4 },
    { "kind": "gradient", "materials": [0, 1], "axis": "z", "from": 0, "to": 1, "warp": 0.05, "seed": 5 },
    { "kind": "noise", "materials": [0, 1], "scale": 0.5, "octaves": 3, "seed": 6 },
    { "kind": "cells", "materials": [0, 2], "size": 0.15, "seed": 7, "border": { "material": 1, "width": 0.02 } },
    { "kind": "speckle", "base": 0, "accents": [1, 2], "size": 0.1, "density": 0.2, "seed": 8 },
    { "kind": "checker", "materials": [0, 1], "size": 0.1 },
    { "kind": "bricks", "materials": [0, 2], "length": 0.2, "height": 0.05, "axis": "z", "seed": 9, "border": { "material": 1, "width": 0.01 } }
  ],
  "steps": [
    { "kind": "add", "name": "add", "shape": 0, "material": 0 },
    { "kind": "carve", "name": "carve", "shape": 1 },
    { "kind": "paint", "name": "paint", "shape": 2, "pattern": 0, "onto": [0] },
    { "kind": "coat", "name": "coat", "pattern": 1, "sides": ["+y", "-x"], "depth": 2, "within": 3, "onto": [0] },
    { "kind": "set", "name": "set", "points": [[0, 0, 0], [0.5, 0, 0]], "size": 0.1, "material": 1 }
  ],
  "objects": [{ "name": "part", "steps": [0, 1, 2, 3, 4] }],
  "nodes": [
    {
      "name": "part",
      "pivot": [0, 0.5, 0],
      "offset": [1, 0, 0],
      "rotation": [{ "axis": "y", "degrees": 120 }, { "axis": "x", "degrees": -35 }],
      "joint": true,
      "childObjects": [0],
      "childNodes": []
    },
    { "name": "model", "childObjects": [], "childNodes": [0] }
  ],
  "rootNodes": [1],
  "names": {
    "shapes3d": { "block": 0 },
    "shapes2d": { "disk": 0 },
    "materials": { "rune": 0, "glass": 1, "stone": 0 },
    "patterns": { "stripes": 0 },
    "steps": { "set": 4 },
    "parts": { "part": 0 }
  }
}
"##;
