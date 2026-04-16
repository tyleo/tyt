import bpy
import math

from mathutils import Matrix, Vector


def reset_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def _apply_fbx_import_compat():
    """Blender 5.0.1 ships an FBX importer whose `blen_read_light` unconditionally
    executes `lamp.cycles.cast_shadow = lamp.use_shadow`, but `cast_shadow` was
    removed from `CyclesLightSettings`, so any FBX containing a light fails to
    import. Rewrite the function source in-place, turning the offending line
    into a no-op. Silently skips when the bug is absent so newer Blender
    releases keep working."""
    import addon_utils
    import importlib
    import inspect
    import os
    import sys
    import textwrap

    addon = next(
        (m for m in addon_utils.modules()
         if getattr(m, "__name__", "") == "io_scene_fbx"),
        None,
    )
    if addon is None or not getattr(addon, "__file__", None):
        return
    addons_parent = os.path.dirname(os.path.dirname(addon.__file__))
    if addons_parent not in sys.path:
        sys.path.insert(0, addons_parent)
    try:
        mod = importlib.import_module("io_scene_fbx.import_fbx")
    except ImportError:
        return
    fn = getattr(mod, "blen_read_light", None)
    if fn is None:
        return
    original = inspect.getsource(fn)
    patched = original.replace(
        "lamp.cycles.cast_shadow = lamp.use_shadow",
        "pass  # tyt-compat: cast_shadow removed from CyclesLightSettings in Blender 5.0",
    )
    if patched == original:
        return
    exec(
        compile(textwrap.dedent(patched), "<tyt-fbx-import-compat>", "exec"),
        mod.__dict__,
    )


def import_fbx(filepath):
    """Imports an FBX via `bpy.ops.import_scene.fbx` after applying any
    Blender version-compatibility shims required by the project."""
    _apply_fbx_import_compat()
    bpy.ops.import_scene.fbx(filepath=filepath)


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


def scene_bounds():
    """Returns (min_corner, max_corner) over all mesh vertices in world space.
    Falls back to a unit box around the origin when no meshes are present."""
    return _mesh_bounds(obj for obj in bpy.data.objects if obj.type == "MESH")


def object_bounds(object_names):
    """Returns (min_corner, max_corner) over the bound_box corners of the named
    objects, transformed to world space. Falls back to scene_bounds() when the
    name list is empty or yields no mesh objects."""
    if not object_names:
        return scene_bounds()
    objects = []
    for name in object_names:
        obj = bpy.data.objects.get(name)
        if obj is not None:
            objects.append(obj)
    if not objects:
        return scene_bounds()
    return _mesh_bounds(objects)


def _mesh_bounds(objects):
    min_corner = Vector((math.inf, math.inf, math.inf))
    max_corner = Vector((-math.inf, -math.inf, -math.inf))
    found = False
    for obj in objects:
        mw = obj.matrix_world
        for corner in obj.bound_box:
            world = mw @ Vector(corner)
            for i in range(3):
                if world[i] < min_corner[i]:
                    min_corner[i] = world[i]
                if world[i] > max_corner[i]:
                    max_corner[i] = world[i]
            found = True
    if not found:
        return Vector((-1.0, -1.0, -1.0)), Vector((1.0, 1.0, 1.0))
    return min_corner, max_corner


def look_at_quaternion(location, target, up):
    """Builds a rotation quaternion so the -Z axis points from `location` to
    `target` with the given `up` hint (Blender camera convention)."""
    forward = (target - location).normalized()
    z_axis = -forward
    x_axis = up.cross(z_axis).normalized()
    y_axis = z_axis.cross(x_axis).normalized()
    rot = Matrix((
        (x_axis.x, y_axis.x, z_axis.x, 0.0),
        (x_axis.y, y_axis.y, z_axis.y, 0.0),
        (x_axis.z, y_axis.z, z_axis.z, 0.0),
        (0.0, 0.0, 0.0, 1.0),
    ))
    return rot.to_quaternion()


def _subject_up(object_names):
    """Returns world-space up vector for orbit: the single matched object's
    local +Z (transformed to world) when exactly one name is given, else
    world +Z."""
    if len(object_names) != 1:
        return Vector((0.0, 0.0, 1.0))
    obj = bpy.data.objects.get(object_names[0])
    if obj is None:
        return Vector((0.0, 0.0, 1.0))
    local_z = obj.matrix_world.to_3x3() @ Vector((0.0, 0.0, 1.0))
    length = local_z.length
    if length < 1.0e-6:
        return Vector((0.0, 0.0, 1.0))
    return local_z / length


def resolve_camera(
    projection,
    lens_mode,
    lens_value,
    resolution_x,
    resolution_y,
    explicit_pose,
    orbit_h_rad,
    orbit_v_rad,
    zoom,
    yaw_rad,
    pitch_rad,
    roll_rad,
    subject_object_names,
):
    """Computes camera position and rotation (quaternion) from either the
    explicit-pose path or the auto-frame path.

    `explicit_pose` is a 6-tuple of Optional floats (pos x/y/z, rot x/y/z in
    RADIANS), or None when auto-framing.

    `subject_object_names` is a list of object names. When non-empty (and
    not explicit-pose), drives bounds / look-at / orbit pivot. When empty,
    scene bounds are used."""
    if explicit_pose is not None:
        px, py, pz, rx, ry, rz = explicit_pose
        cam_pos = Vector((px or 0.0, py or 0.0, pz or 0.0))
        euler_matrix = Matrix.Identity(3)
        euler_matrix = (
            Matrix.Rotation(rz or 0.0, 3, "Z")
            @ Matrix.Rotation(ry or 0.0, 3, "Y")
            @ Matrix.Rotation(rx or 0.0, 3, "X")
        )
        cam_quat = euler_matrix.to_quaternion()
        return cam_pos, cam_quat

    min_c, max_c = object_bounds(subject_object_names)
    center = (min_c + max_c) * 0.5
    diagonal = (max_c - min_c).length
    aspect = max(resolution_x, 1) / max(resolution_y, 1)
    fit_size = diagonal / max(min(1.0, aspect), 1.0e-6)

    if projection == "PERSP":
        if lens_mode == "fov":
            half_fov = math.radians(lens_value) * 0.5
            distance = (fit_size * 0.5) / max(math.tan(half_fov), 1.0e-6)
        else:
            sensor_width = 36.0
            distance = (fit_size * lens_value) / sensor_width
        distance = max(distance, diagonal * 1.5, 2.0)
    else:
        distance = max(diagonal * 1.5, 2.0)
    distance *= zoom

    subject_up = _subject_up(subject_object_names)
    direction = Vector((1.0, -1.0, 0.7)).normalized()
    if orbit_h_rad != 0.0:
        direction = Matrix.Rotation(orbit_h_rad, 3, subject_up) @ direction
    horizontal_axis = direction.cross(subject_up)
    if horizontal_axis.length >= 1.0e-6:
        horizontal_axis.normalize()
        if orbit_v_rad != 0.0:
            direction = Matrix.Rotation(orbit_v_rad, 3, horizontal_axis) @ direction

    cam_pos = center + direction * distance
    base_rot_matrix = look_at_quaternion(cam_pos, center, subject_up).to_matrix()

    local_rot = (
        Matrix.Rotation(yaw_rad, 3, "Y")
        @ Matrix.Rotation(pitch_rad, 3, "X")
        @ Matrix.Rotation(roll_rad, 3, "Z")
    )
    final_rot = base_rot_matrix @ local_rot
    return cam_pos, final_rot.to_quaternion()
