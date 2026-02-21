import bpy
import sys

from common import strip_all_materials, deselect_all, export_fbx


def extract_single_mesh(mesh_name: str, output_mesh_name: str):
    # Ensure we're in Object mode for ops
    if bpy.ops.object.mode_set.poll():
        bpy.ops.object.mode_set(mode="OBJECT")

    target = bpy.data.objects.get(mesh_name)
    if target is None:
        raise ValueError(f"Mesh '{mesh_name}' not found.")

    if target.type != "MESH":
        raise ValueError(f"Object '{mesh_name}' is type '{target.type}', expected 'MESH'.")

    # Deselect all
    deselect_all()

    # Unparent target (keep world transform)
    target.select_set(True)
    bpy.context.view_layer.objects.active = target
    if target.parent is not None:
        if bpy.ops.object.parent_clear.poll():
            bpy.ops.object.parent_clear(type="CLEAR_KEEP_TRANSFORM")
        target.parent = None  # extra safety

    # Delete everything else
    others = [o for o in bpy.data.objects if o != target]
    for o in others:
        bpy.data.objects.remove(o, do_unlink=True)

    # Purge orphan datablocks (may require multiple passes)
    try:
        for _ in range(5):
            bpy.ops.outliner.orphans_purge(
                do_local_ids=True, do_linked_ids=True, do_recursive=True
            )
    except Exception:
        pass  # safe to skip headless / missing UI context

    # Rename
    target.name = output_mesh_name
    if target.data:
        target.data.name = output_mesh_name

    return target


def parse_args():
    argv = sys.argv
    if "--" not in argv:
        raise SystemExit(
            "Usage: blender -b --python script.py -- <input_fbx> <mesh_name> <output_fbx> <output_mesh_name>"
        )

    tokens = argv[argv.index("--") + 1 :]
    if len(tokens) != 4:
        raise SystemExit(
            "Expected 4 args: <input_fbx> <mesh_name> <output_fbx> <output_mesh_name>"
        )

    input_fbx, mesh_name, output_fbx, output_mesh_name = tokens
    return input_fbx, mesh_name, output_fbx, output_mesh_name


def main():
    input_fbx, mesh_name, output_fbx, output_mesh_name = parse_args()

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.fbx(filepath=input_fbx)

    strip_all_materials()
    extract_single_mesh(mesh_name, output_mesh_name)

    export_fbx(output_fbx)

if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
