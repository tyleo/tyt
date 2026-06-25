import bpy
import sys

from common import export_fbx, import_fbx


def clear_materials_on(object_names):
    for name in object_names:
        obj = bpy.data.objects.get(name)
        if obj is None:
            raise ValueError(f"Object '{name}' not found.")
        if obj.type != "MESH":
            continue
        obj.data.materials.clear()
    for mat in list(bpy.data.materials):
        if mat.users == 0:
            bpy.data.materials.remove(mat)


def parse_args():
    argv = sys.argv
    if "--" not in argv:
        raise SystemExit(
            "Usage: blender -b --python script.py -- <input_fbx> <output_fbx> "
            "<clear_materials> <num_objects> <obj_name_1> ... <obj_name_N>"
        )

    tokens = argv[argv.index("--") + 1 :]
    if len(tokens) < 4:
        raise SystemExit(
            "Expected at least 4 args: <input_fbx> <output_fbx> <clear_materials> <num_objects> [<obj_name>...]"
        )

    input_fbx = tokens[0]
    output_fbx = tokens[1]
    clear_materials = tokens[2] == "true"
    num_objects = int(tokens[3])

    expected = 4 + num_objects
    if len(tokens) != expected:
        raise SystemExit(
            f"Expected {expected} args for <num_objects>={num_objects}, got {len(tokens)}"
        )

    object_names = tokens[4:]
    return input_fbx, output_fbx, clear_materials, object_names


def main():
    input_fbx, output_fbx, clear_materials, object_names = parse_args()

    bpy.ops.wm.read_factory_settings(use_empty=True)
    import_fbx(input_fbx)

    if clear_materials:
        clear_materials_on(object_names)

    export_fbx(output_fbx)


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
