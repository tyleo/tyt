import bpy
import sys

from mathutils import Matrix

from common import strip_all_materials, export_fbx


def parse_optional_float(token):
    if token == "none":
        return None
    return float(token)


def apply_local(obj, set_pos, set_rot, set_scl, mod_pos, mod_rot, mod_scl):
    for i in range(3):
        if set_pos[i] is not None:
            obj.location[i] = set_pos[i]
        if mod_pos[i] is not None:
            obj.location[i] += mod_pos[i]

        if set_rot[i] is not None:
            obj.rotation_euler[i] = set_rot[i]
        if mod_rot[i] is not None:
            obj.rotation_euler[i] += mod_rot[i]

        if set_scl[i] is not None:
            obj.scale[i] = set_scl[i]
        if mod_scl[i] is not None:
            obj.scale[i] += mod_scl[i]


def apply_world(obj, set_pos, set_rot, set_scl, mod_pos, mod_rot, mod_scl):
    world = obj.matrix_world
    loc = world.to_translation()
    rot = world.to_euler()
    scl = world.to_scale()

    for i in range(3):
        if set_pos[i] is not None:
            loc[i] = set_pos[i]
        if mod_pos[i] is not None:
            loc[i] += mod_pos[i]

        if set_rot[i] is not None:
            rot[i] = set_rot[i]
        if mod_rot[i] is not None:
            rot[i] += mod_rot[i]

        if set_scl[i] is not None:
            scl[i] = set_scl[i]
        if mod_scl[i] is not None:
            scl[i] += mod_scl[i]

    obj.matrix_world = Matrix.LocRotScale(loc, rot, scl)


def apply_transforms(
    object_names,
    set_pos, set_rot, set_scl,
    mod_pos, mod_rot, mod_scl,
    space,
):
    for name in object_names:
        obj = bpy.data.objects.get(name)
        if obj is None:
            raise ValueError(f"Object '{name}' not found.")

        if space == "world":
            apply_world(obj, set_pos, set_rot, set_scl, mod_pos, mod_rot, mod_scl)
        else:
            apply_local(obj, set_pos, set_rot, set_scl, mod_pos, mod_rot, mod_scl)


def parse_args():
    argv = sys.argv
    if "--" not in argv:
        raise SystemExit(
            "Usage: blender -b --python script.py -- <input_fbx> <output_fbx> "
            "<set_pos_x> <set_pos_y> <set_pos_z> "
            "<set_rot_x> <set_rot_y> <set_rot_z> "
            "<set_scl_x> <set_scl_y> <set_scl_z> "
            "<mod_pos_x> <mod_pos_y> <mod_pos_z> "
            "<mod_rot_x> <mod_rot_y> <mod_rot_z> "
            "<mod_scl_x> <mod_scl_y> <mod_scl_z> "
            "<space> <num_objects> <obj_name_1> ... <obj_name_N>"
        )

    tokens = argv[argv.index("--") + 1 :]
    if len(tokens) < 22:
        raise SystemExit(
            "Expected at least 22 args: <input_fbx> <output_fbx> "
            "<9 set slots> <9 mod slots> <space> <num_objects> [<obj_name>...]"
        )

    input_fbx = tokens[0]
    output_fbx = tokens[1]
    slots = [parse_optional_float(t) for t in tokens[2:20]]
    set_pos = slots[0:3]
    set_rot = slots[3:6]
    set_scl = slots[6:9]
    mod_pos = slots[9:12]
    mod_rot = slots[12:15]
    mod_scl = slots[15:18]
    space = tokens[20]
    if space not in ("default", "local", "world"):
        raise SystemExit(f"Expected space in (default, local, world), got {space!r}")
    num_objects = int(tokens[21])

    expected = 22 + num_objects
    if len(tokens) != expected:
        raise SystemExit(
            f"Expected {expected} args for <num_objects>={num_objects}, got {len(tokens)}"
        )

    object_names = tokens[22:]
    return (
        input_fbx, output_fbx,
        set_pos, set_rot, set_scl,
        mod_pos, mod_rot, mod_scl,
        space, object_names,
    )


def main():
    (
        input_fbx, output_fbx,
        set_pos, set_rot, set_scl,
        mod_pos, mod_rot, mod_scl,
        space, object_names,
    ) = parse_args()

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.fbx(filepath=input_fbx)

    strip_all_materials()
    apply_transforms(
        object_names,
        set_pos, set_rot, set_scl,
        mod_pos, mod_rot, mod_scl,
        space,
    )

    export_fbx(output_fbx)


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
