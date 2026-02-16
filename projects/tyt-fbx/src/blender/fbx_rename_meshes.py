import bpy
import sys

from common import strip_all_materials, export_fbx


def rename_meshes(new_name):
    """
    Rename mesh object(s) and their mesh datablocks.

    Behavior:
    - If exactly one MESH object exists: rename it to <new_name>
    - If multiple MESH objects exist: rename them to <new_name>-001, <new_name>-002, ...
    """
    mesh_objs = [o for o in bpy.data.objects if o.type == "MESH"]
    if not mesh_objs:
        return

    if len(mesh_objs) == 1:
        obj = mesh_objs[0]
        obj.name = new_name
        if obj.data:
            obj.data.name = new_name
        return

    for i, obj in enumerate(sorted(mesh_objs, key=lambda o: o.name), start=1):
        numbered_name = f"{new_name}-{i:03d}"
        obj.name = numbered_name
        if obj.data:
            obj.data.name = numbered_name


def parse_args():
    argv = sys.argv
    if "--" not in argv:
        raise SystemExit(
            "Usage: blender -b --python script.py -- <input_fbx> <output_fbx> <output_mesh_name>"
        )

    tokens = argv[argv.index("--") + 1 :]
    if len(tokens) != 3:
        raise SystemExit(
            "Expected 3 args: <input_fbx> <output_fbx> <output_mesh_name>"
        )

    input_fbx, output_fbx, output_mesh_name = tokens
    return input_fbx, output_fbx, output_mesh_name


def main():
    input_fbx, output_fbx, output_mesh_name = parse_args()

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.fbx(filepath=input_fbx)

    strip_all_materials()
    rename_meshes(output_mesh_name)

    export_fbx(output_fbx)


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
