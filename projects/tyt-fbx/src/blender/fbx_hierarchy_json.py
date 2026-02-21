import bpy
import json
import sys


def build_hierarchy_json():
    result = []

    def visit(obj, parent_path):
        if parent_path:
            path = f"{parent_path}/{obj.name}"
        else:
            path = obj.name
        result.append({"name": obj.name, "path": path, "type": obj.type})
        for child in sorted(obj.children, key=lambda c: c.name):
            visit(child, path)

    roots = sorted(
        [o for o in bpy.data.objects if o.parent is None],
        key=lambda o: o.name,
    )

    for root in roots:
        visit(root, "")

    return result


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

    hierarchy = build_hierarchy_json()
    print(json.dumps(hierarchy))


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
