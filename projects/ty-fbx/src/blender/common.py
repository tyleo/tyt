import bpy


def reset_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)


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


def deselect_all():
    for o in bpy.context.view_layer.objects:
        o.select_set(False)


def export_fbx(filepath):
    bpy.ops.export_scene.fbx(
        filepath=filepath,
        path_mode="STRIP",
        embed_textures=False,
        add_leaf_bones=False,
        bake_space_transform=False,
        use_space_transform=True,
    )
