import bpy
import sys

def strip_all_materials():
    """
    Remove all materials from mesh material slots, then delete orphaned material datablocks.
    """
    for obj in bpy.data.objects:
        if obj.type != "MESH":
            continue
        obj.data.materials.clear()

    for mat in list(bpy.data.materials):
        if mat.users == 0:
            bpy.data.materials.remove(mat)

def extract_single_mesh(parent_mesh_name: str, output_mesh_name: str):
    # Ensure we're in Object mode for ops
    if bpy.ops.object.mode_set.poll():
        bpy.ops.object.mode_set(mode="OBJECT")

    root = bpy.data.objects.get(parent_mesh_name)
    if root is None:
        raise ValueError(f"Parent '{parent_mesh_name}' not found.")

    # First direct child mesh (no recursion)
    target = next((c for c in root.children if c.type == "MESH"), None)
    if target is None:
        raise ValueError(f"No direct child mesh found under '{parent_mesh_name}'.")

    # Deselect all
    for o in bpy.context.view_layer.objects:
        o.select_set(False)

    # Unparent target (keep world transform)
    target.select_set(True)
    bpy.context.view_layer.objects.active = target
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
            "Usage: blender -b --python script.py -- <input_fbx> <parent_mesh_name> <output_fbx> <output_mesh_name>"
        )

    tokens = argv[argv.index("--") + 1 :]
    if len(tokens) != 4:
        raise SystemExit(
            "Expected 4 args: <input_fbx> <parent_mesh_name> <output_fbx> <output_mesh_name>"
        )

    input_fbx, parent_mesh_name, output_fbx, output_mesh_name = tokens
    return input_fbx, parent_mesh_name, output_fbx, output_mesh_name


def main():
    input_fbx, parent_mesh_name, output_fbx, output_mesh_name = parse_args()

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.fbx(filepath=input_fbx)

    strip_all_materials()
    extract_single_mesh(parent_mesh_name, output_mesh_name)

    bpy.ops.export_scene.fbx(
        filepath=output_fbx,
        path_mode="STRIP",
        embed_textures=False,
        add_leaf_bones=False,
        bake_space_transform=False,
        use_space_transform=True,
    )


if __name__ == "__main__":
    main()