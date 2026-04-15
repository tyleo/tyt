import bpy
import math
import sys


def format_vec(vec, precision):
    components = [f"{v:.{precision}f}" for v in (vec[0], vec[1], vec[2])]
    width = max(len(c) for c in components)
    x, y, z = (c.rjust(width) for c in components)
    return f'{{ "X": {x}, "Y": {y}, "Z": {z} }}'


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


def print_hierarchy(obj, prefix, is_last, precision, is_world, is_degrees):
    connector = "\u2514 " if is_last else "\u251c "
    print(f"{prefix}{connector}{obj.name} ({obj.type})")
    extension = "  " if is_last else "\u2502 "
    child_prefix = prefix + extension
    children = sorted(obj.children, key=lambda c: c.name)
    if precision is not None:
        transform_is_last = len(children) == 0
        print_transform(
            obj, child_prefix, transform_is_last, precision, is_world, is_degrees
        )
    for i, child in enumerate(children):
        print_hierarchy(
            child, child_prefix, i == len(children) - 1, precision, is_world, is_degrees
        )


def parse_bool(token):
    if token == "true":
        return True
    if token == "false":
        return False
    raise SystemExit(f"Expected 'true' or 'false', got {token!r}")


def parse_args():
    argv = sys.argv
    if "--" not in argv:
        raise SystemExit(
            "Usage: blender -b --python script.py -- "
            "<input_fbx> [<precision> <is_world> <is_degrees>]"
        )

    tokens = argv[argv.index("--") + 1 :]
    if len(tokens) == 1:
        return tokens[0], None, False, False
    if len(tokens) == 4:
        return (
            tokens[0],
            int(tokens[1]),
            parse_bool(tokens[2]),
            parse_bool(tokens[3]),
        )
    raise SystemExit(
        "Expected 1 or 4 args: <input_fbx> [<precision> <is_world> <is_degrees>]"
    )


def main():
    input_fbx, precision, is_world, is_degrees = parse_args()

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.fbx(filepath=input_fbx)

    roots = sorted(
        [o for o in bpy.data.objects if o.parent is None],
        key=lambda o: o.name,
    )

    for i, root in enumerate(roots):
        print_hierarchy(root, "", i == len(roots) - 1, precision, is_world, is_degrees)


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
