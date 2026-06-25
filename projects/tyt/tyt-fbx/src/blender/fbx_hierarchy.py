import bpy
import math
import sys

from common import import_fbx
from mathutils import Vector


def format_vec(vec, precision):
    x, y, z = (f"{v:.{precision}f}" for v in (vec[0], vec[1], vec[2]))
    return f'{{ "X": {x}, "Y": {y}, "Z": {z} }}'


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
        "<show_xfm> <xfm_prec> <xfm_world> <xfm_deg> "
        "<show_bnd> <bnd_prec> <bnd_world> <bnd_scale> "
        "<show_ext> <ext_prec> <ext_world> <ext_scale> "
        "<collapse_ancestors> <collapse_descendants> "
        "<num_paths> <path_1> ... <path_n>"
    )
    if "--" not in argv:
        raise SystemExit(f"Usage: blender -b --python script.py -- {schema}")

    tokens = argv[argv.index("--") + 1 :]
    if len(tokens) < 16:
        raise SystemExit(f"Expected at least 16 args ({schema}), got {len(tokens)}")

    num_paths = int(tokens[15])
    expected = 16 + num_paths
    if len(tokens) != expected:
        raise SystemExit(f"Expected {expected} args ({schema}), got {len(tokens)}")

    return {
        "input_fbx":            tokens[0],
        "show_transforms":      parse_bool(tokens[1]),
        "xfm_prec":             int(tokens[2]),
        "xfm_world":            parse_bool(tokens[3]),
        "xfm_degrees":          parse_bool(tokens[4]),
        "show_bounds":          parse_bool(tokens[5]),
        "bnd_prec":             int(tokens[6]),
        "bnd_world":            parse_bool(tokens[7]),
        "bnd_scale":            parse_bool(tokens[8]),
        "show_extents":         parse_bool(tokens[9]),
        "ext_prec":             int(tokens[10]),
        "ext_world":            parse_bool(tokens[11]),
        "ext_scale":            parse_bool(tokens[12]),
        "collapse_ancestors":   parse_bool(tokens[13]),
        "collapse_descendants": parse_bool(tokens[14]),
        "matched_paths":        list(tokens[16 : 16 + num_paths]),
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


def print_transform(obj, prefix, is_last, precision, is_world, is_degrees):
    connector = "\u2514 " if is_last else "\u251c "
    print(f"{prefix}{connector}(TRANSFORM)")
    extension = "  " if is_last else "\u2502 "
    child_prefix = prefix + extension
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
    lines = [
        (format_vec(location, precision), "POSITION"),
        (format_vec(rotation, precision), "ROTATION"),
        (format_vec(scale, precision), "SCALE"),
    ]
    for i, (vec_json, label) in enumerate(lines):
        child_connector = "\u2514 " if i == len(lines) - 1 else "\u251c "
        print(f"{child_prefix}{child_connector}{vec_json} ({label})")


def print_extents(obj, prefix, is_last, precision, is_world, apply_scale):
    aabb = compute_aabb(obj, is_world, apply_scale)
    if aabb is None:
        return
    min_v, max_v = aabb
    extents = Vector((max_v.x - min_v.x, max_v.y - min_v.y, max_v.z - min_v.z))
    connector = "\u2514 " if is_last else "\u251c "
    print(f"{prefix}{connector}{format_vec(extents, precision)} (EXTENTS)")


def print_bounds(obj, prefix, is_last, precision, is_world, apply_scale):
    aabb = compute_aabb(obj, is_world, apply_scale)
    if aabb is None:
        return
    min_v, max_v = aabb
    connector = "\u2514 " if is_last else "\u251c "
    print(f"{prefix}{connector}(BOUNDS)")
    extension = "  " if is_last else "\u2502 "
    child_prefix = prefix + extension

    axes = [
        ("X", min_v.x, max_v.x),
        ("Y", min_v.y, max_v.y),
        ("Z", min_v.z, max_v.z),
    ]
    for i, (axis, lo, hi) in enumerate(axes):
        child_connector = "\u2514 " if i == len(axes) - 1 else "\u251c "
        lo_s = f"{lo:.{precision}f}"
        hi_s = f"{hi:.{precision}f}"
        print(
            f'{child_prefix}{child_connector}'
            f'{{ "Min{axis}": {lo_s}, "Max{axis}": {hi_s} }} ({axis}-BOUNDS)'
        )


def object_has_geometry(obj):
    if obj.type == "MESH" and obj.data is not None:
        return True
    for child in obj.children:
        if object_has_geometry(child):
            return True
    return False


def print_hierarchy(obj, path, prefix, is_last, opts,
                    matched_set, ancestor_set, in_match):
    connector = "\u2514 " if is_last else "\u251c "
    print(f"{prefix}{connector}{obj.name} ({obj.type})")
    extension = "  " if is_last else "\u2502 "
    child_prefix = prefix + extension

    is_matched = matched_set is not None and path in matched_set
    now_in_match = in_match or is_matched

    if now_in_match and opts["collapse_descendants"] and obj.children:
        children = []
        show_descendants_marker = True
    elif matched_set is None or now_in_match:
        children = sorted(obj.children, key=lambda c: c.name)
        show_descendants_marker = False
    else:
        allowed = matched_set | ancestor_set
        children = sorted(
            [c for c in obj.children if f"{path}/{c.name}" in allowed],
            key=lambda c: c.name,
        )
        show_descendants_marker = False

    has_geom = object_has_geometry(obj)
    subtrees = []
    if opts["show_transforms"]:
        subtrees.append("transform")
    if opts["show_extents"] and has_geom:
        subtrees.append("extents")
    if opts["show_bounds"] and has_geom:
        subtrees.append("bounds")
    if show_descendants_marker:
        subtrees.append("descendants")

    total = len(subtrees) + len(children)

    for i, kind in enumerate(subtrees):
        is_sub_last = i == total - 1
        if kind == "descendants":
            sub_connector = "\u2514 " if is_sub_last else "\u251c "
            print(f"{child_prefix}{sub_connector}(DESCENDANTS)")
        elif kind == "transform":
            print_transform(
                obj, child_prefix, is_sub_last,
                opts["xfm_prec"], opts["xfm_world"], opts["xfm_degrees"],
            )
        elif kind == "extents":
            print_extents(
                obj, child_prefix, is_sub_last,
                opts["ext_prec"], opts["ext_world"], opts["ext_scale"],
            )
        else:
            print_bounds(
                obj, child_prefix, is_sub_last,
                opts["bnd_prec"], opts["bnd_world"], opts["bnd_scale"],
            )

    for i, child in enumerate(children):
        is_child_last = (len(subtrees) + i) == total - 1
        child_path = f"{path}/{child.name}"
        print_hierarchy(
            child, child_path, child_prefix, is_child_last, opts,
            matched_set, ancestor_set, now_in_match,
        )


def print_collapsed_match(path, obj, prefix, is_last, opts):
    has_ancestors = "/" in path
    if has_ancestors:
        connector = "\u2514 " if is_last else "\u251c "
        print(f"{prefix}{connector}(ANCESTORS)")
        extension = "  " if is_last else "\u2502 "
        child_prefix = prefix + extension
        print_hierarchy(
            obj, path, child_prefix, True, opts,
            None, None, True,
        )
    else:
        print_hierarchy(
            obj, path, prefix, is_last, opts,
            None, None, True,
        )


def collect_ancestor_paths(paths):
    ancestors = set()
    for p in paths:
        parts = p.split("/")
        for i in range(1, len(parts)):
            ancestors.add("/".join(parts[:i]))
    return ancestors


def build_path_map(roots):
    pmap = {}

    def visit(obj, parent_path):
        path = f"{parent_path}/{obj.name}" if parent_path else obj.name
        pmap[path] = obj
        for child in sorted(obj.children, key=lambda c: c.name):
            visit(child, path)

    for root in roots:
        visit(root, "")
    return pmap


def main():
    opts = parse_args()

    bpy.ops.wm.read_factory_settings(use_empty=True)
    import_fbx(opts["input_fbx"])

    roots = sorted(
        [o for o in bpy.data.objects if o.parent is None],
        key=lambda o: o.name,
    )

    matched_paths = opts["matched_paths"]

    if not matched_paths:
        for i, root in enumerate(roots):
            print_hierarchy(
                root, root.name, "", i == len(roots) - 1, opts,
                None, None, False,
            )
        return

    if opts["collapse_ancestors"]:
        pmap = build_path_map(roots)
        visible = [(p, pmap[p]) for p in matched_paths if p in pmap]
        for i, (p, obj) in enumerate(visible):
            print_collapsed_match(p, obj, "", i == len(visible) - 1, opts)
    else:
        matched_set = set(matched_paths)
        ancestor_set = collect_ancestor_paths(matched_paths)
        allowed_roots = matched_set | ancestor_set
        visible_roots = [r for r in roots if r.name in allowed_roots]
        for i, root in enumerate(visible_roots):
            print_hierarchy(
                root, root.name, "", i == len(visible_roots) - 1, opts,
                matched_set, ancestor_set, False,
            )


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
