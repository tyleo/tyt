import bpy
import json
import math
import sys

from common import import_fbx
from mathutils import Vector


def parse_bool(token):
    if token == "true":
        return True
    if token == "false":
        return False
    raise SystemExit(f"Expected 'true' or 'false', got {token!r}")


def parse_args():
    argv = sys.argv
    schema = (
        "<input_fbx> "
        "[<show_xfm> <xfm_prec> <xfm_world> <xfm_deg> "
        "<show_bnd> <bnd_prec> <bnd_world> <bnd_scale> "
        "<show_ext> <ext_prec> <ext_world> <ext_scale>]"
    )
    if "--" not in argv:
        raise SystemExit(f"Usage: blender -b --python script.py -- {schema}")

    tokens = argv[argv.index("--") + 1 :]
    if len(tokens) == 1:
        tokens = tokens + ["false", "2", "false", "false"] * 3
    if len(tokens) != 13:
        raise SystemExit(f"Expected 1 or 13 args ({schema}), got {len(tokens)}")

    return {
        "input_fbx":       tokens[0],
        "show_transforms": parse_bool(tokens[1]),
        "xfm_prec":        int(tokens[2]),
        "xfm_world":       parse_bool(tokens[3]),
        "xfm_degrees":     parse_bool(tokens[4]),
        "show_bounds":     parse_bool(tokens[5]),
        "bnd_prec":        int(tokens[6]),
        "bnd_world":       parse_bool(tokens[7]),
        "bnd_scale":       parse_bool(tokens[8]),
        "show_extents":    parse_bool(tokens[9]),
        "ext_prec":        int(tokens[10]),
        "ext_world":       parse_bool(tokens[11]),
        "ext_scale":       parse_bool(tokens[12]),
    }


def collect_world_corners(obj):
    corners = []
    if obj.type == "MESH" and obj.data is not None:
        mw = obj.matrix_world
        for c in obj.bound_box:
            corners.append(mw @ Vector(c))
    for child in obj.children:
        corners.extend(collect_world_corners(child))
    return corners


def compute_aabb(obj, is_world, apply_scale):
    world_corners = collect_world_corners(obj)
    if not world_corners:
        return None
    if is_world:
        final = world_corners
    else:
        inv = obj.matrix_world.inverted()
        final = [inv @ c for c in world_corners]
        if apply_scale:
            sx, sy, sz = obj.scale
            final = [Vector((v.x * sx, v.y * sy, v.z * sz)) for v in final]
    min_v = Vector((
        min(v.x for v in final),
        min(v.y for v in final),
        min(v.z for v in final),
    ))
    max_v = Vector((
        max(v.x for v in final),
        max(v.y for v in final),
        max(v.z for v in final),
    ))
    return (min_v, max_v)


def format_components(vec, precision):
    return [f"{vec[i]:.{precision}f}" for i in range(3)]


def transform_payload(obj, precision, is_world, is_degrees):
    if is_world:
        matrix = obj.matrix_world
        location = matrix.to_translation()
        rotation = matrix.to_euler()
        scale = matrix.to_scale()
    else:
        location = obj.location
        rotation = obj.rotation_euler
        scale = obj.scale
    if is_degrees:
        rotation = [math.degrees(c) for c in rotation]
    return {
        "position": format_components(location, precision),
        "rotation": format_components(rotation, precision),
        "scale": format_components(scale, precision),
    }


def bounds_payload(obj, precision, is_world, apply_scale):
    aabb = compute_aabb(obj, is_world, apply_scale)
    if aabb is None:
        return None
    min_v, max_v = aabb
    return {
        "min": format_components(min_v, precision),
        "max": format_components(max_v, precision),
    }


def extents_payload(obj, precision, is_world, apply_scale):
    aabb = compute_aabb(obj, is_world, apply_scale)
    if aabb is None:
        return None
    min_v, max_v = aabb
    extents = Vector((max_v.x - min_v.x, max_v.y - min_v.y, max_v.z - min_v.z))
    return format_components(extents, precision)


def build_hierarchy_json(opts):
    result = []

    def visit(obj, parent_path):
        if parent_path:
            path = f"{parent_path}/{obj.name}"
        else:
            path = obj.name
        entry = {"name": obj.name, "path": path, "type": obj.type}
        if opts["show_transforms"]:
            entry["transform"] = transform_payload(
                obj, opts["xfm_prec"], opts["xfm_world"], opts["xfm_degrees"],
            )
        if opts["show_bounds"]:
            bounds = bounds_payload(
                obj, opts["bnd_prec"], opts["bnd_world"], opts["bnd_scale"],
            )
            if bounds is not None:
                entry["bounds"] = bounds
        if opts["show_extents"]:
            extents = extents_payload(
                obj, opts["ext_prec"], opts["ext_world"], opts["ext_scale"],
            )
            if extents is not None:
                entry["extents"] = extents
        result.append(entry)
        for child in sorted(obj.children, key=lambda c: c.name):
            visit(child, path)

    roots = sorted(
        [o for o in bpy.data.objects if o.parent is None],
        key=lambda o: o.name,
    )

    for root in roots:
        visit(root, "")

    return result


def main():
    opts = parse_args()

    bpy.ops.wm.read_factory_settings(use_empty=True)
    import_fbx(opts["input_fbx"])

    hierarchy = build_hierarchy_json(opts)
    print(json.dumps(hierarchy))


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
