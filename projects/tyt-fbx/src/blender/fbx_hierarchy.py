import bpy
import sys


def print_hierarchy(obj, prefix="", is_last=True):
    connector = "\u2514 " if is_last else "\u251c "
    print(f"{prefix}{connector}{obj.name} ({obj.type})")
    children = sorted(obj.children, key=lambda c: c.name)
    for i, child in enumerate(children):
        extension = "  " if is_last else "\u2502 "
        print_hierarchy(child, prefix + extension, i == len(children) - 1)


def parse_args():
    argv = sys.argv
    if "--" not in argv:
        raise SystemExit(
            "Usage: blender -b --python script.py -- <input_fbx>"
        )

    tokens = argv[argv.index("--") + 1 :]
    if len(tokens) != 1:
        raise SystemExit("Expected 1 arg: <input_fbx>")

    return tokens[0]


def main():
    input_fbx = parse_args()

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.fbx(filepath=input_fbx)

    roots = sorted(
        [o for o in bpy.data.objects if o.parent is None],
        key=lambda o: o.name,
    )

    for i, root in enumerate(roots):
        is_last = i == len(roots) - 1
        connector = "\u2514 " if is_last else "\u251c "
        print(f"{connector}{root.name} ({root.type})")
        children = sorted(root.children, key=lambda c: c.name)
        for j, child in enumerate(children):
            extension = "  " if is_last else "\u2502 "
            print_hierarchy(child, extension, j == len(children) - 1)


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
