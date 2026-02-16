import bpy
import sys

from common import strip_all_materials, deselect_all, export_fbx


def delete_empty_objects_without_children():
    empties = [o for o in bpy.data.objects if o.type == "EMPTY" and len(o.children) == 0]
    if not empties:
        return

    deselect_all()
    for o in empties:
        o.select_set(True)
    bpy.ops.object.delete()


def reduce_to_single_mesh(new_name):
    """
    Collapse all mesh objects into one mesh object.
    - Clears parenting while keeping world transforms (so join doesn't lose placement).
    - Deletes now-unused empties.
    - Joins meshes.
    - Renames joined object + datablock to <new_name>.
    """
    if bpy.ops.object.mode_set.poll():
        bpy.ops.object.mode_set(mode="OBJECT")

    mesh_objs = [o for o in bpy.data.objects if o.type == "MESH"]
    if not mesh_objs:
        return

    # Clear parenting but keep transforms
    deselect_all()
    for o in mesh_objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = mesh_objs[0]
    if bpy.ops.object.parent_clear.poll():
        bpy.ops.object.parent_clear(type="CLEAR_KEEP_TRANSFORM")

    # Remove empties that are now unused
    delete_empty_objects_without_children()

    # Refresh list after parent clear
    mesh_objs = [o for o in bpy.data.objects if o.type == "MESH"]
    if not mesh_objs:
        return

    # Join all meshes into the active one
    deselect_all()
    for o in mesh_objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = mesh_objs[0]
    bpy.ops.object.join()

    joined = bpy.context.view_layer.objects.active
    if not joined or joined.type != "MESH":
        return

    # Apply transforms so the final object has identity transforms
    deselect_all()
    joined.select_set(True)
    bpy.context.view_layer.objects.active = joined

    # Rename final object + datablock
    joined.name = new_name
    if joined.data:
        joined.data.name = new_name


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
    reduce_to_single_mesh(output_mesh_name)

    export_fbx(output_fbx)


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
