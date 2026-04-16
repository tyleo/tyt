import bpy
import sys

from common import strip_all_materials, export_fbx, import_fbx


def rename_objects(pairs):
    for old_name, new_name in pairs:
        obj = bpy.data.objects.get(old_name)
        if obj is None:
            raise ValueError(f"Object '{old_name}' not found.")
        obj.name = new_name
        if obj.data is not None:
            obj.data.name = new_name


def parse_args():
    argv = sys.argv
    if "--" not in argv:
        raise SystemExit(
            "Usage: blender -b --python script.py -- <input_fbx> <output_fbx> "
            "<num_pairs> <old_1> <new_1> ... <old_N> <new_N>"
        )

    tokens = argv[argv.index("--") + 1 :]
    if len(tokens) < 3:
        raise SystemExit(
            "Expected at least 3 args: <input_fbx> <output_fbx> <num_pairs> [<old> <new>...]"
        )

    input_fbx = tokens[0]
    output_fbx = tokens[1]
    num_pairs = int(tokens[2])

    expected = 3 + 2 * num_pairs
    if len(tokens) != expected:
        raise SystemExit(
            f"Expected {expected} args for <num_pairs>={num_pairs}, got {len(tokens)}"
        )

    rest = tokens[3:]
    pairs = list(zip(rest[0::2], rest[1::2]))
    return input_fbx, output_fbx, pairs


def main():
    input_fbx, output_fbx, pairs = parse_args()

    bpy.ops.wm.read_factory_settings(use_empty=True)
    import_fbx(input_fbx)

    strip_all_materials()
    rename_objects(pairs)

    export_fbx(output_fbx)


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
