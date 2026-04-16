import bpy
import math
import sys
import warnings

from mathutils import Vector

from common import (
    import_fbx,
    look_at_quaternion,
    reset_scene,
    resolve_camera,
    scene_bounds,
)


def _parse_optional_float(token):
    return None if token == "none" else float(token)


def parse_args():
    argv = sys.argv
    if "--" not in argv:
        raise SystemExit(
            "Usage: blender -b --python script.py -- <input_fbx> <output_png> "
            "<resolution_x> <resolution_y> "
            "<projection> <lens_mode> <lens_value> <ortho_scale> "
            "<near> <far> "
            "<renderer> <samples> "
            "<lighting> "
            "<cam_pos_x> <cam_pos_y> <cam_pos_z> "
            "<cam_rot_x_rad> <cam_rot_y_rad> <cam_rot_z_rad> "
            "<orbit_h_rad> <orbit_v_rad> <zoom> "
            "<yaw_rad> <pitch_rad> <roll_rad> "
            "<emit_camera> "
            "<num_subject_objects> <subject_object_names...>"
        )

    tokens = argv[argv.index("--") + 1 :]
    if len(tokens) < 27:
        raise SystemExit(f"Expected at least 27 args, got {len(tokens)}")

    input_fbx = tokens[0]
    output_png = tokens[1]
    resolution_x = int(tokens[2])
    resolution_y = int(tokens[3])
    projection = tokens[4]
    lens_mode = tokens[5]
    lens_value = float(tokens[6])
    ortho_scale = float(tokens[7])
    near = float(tokens[8])
    far = float(tokens[9])
    renderer = tokens[10]
    samples = int(tokens[11])
    lighting = tokens[12]

    explicit_slots = tuple(_parse_optional_float(t) for t in tokens[13:19])
    any_explicit = any(v is not None for v in explicit_slots)
    explicit_pose = explicit_slots if any_explicit else None

    orbit_h_rad = float(tokens[19])
    orbit_v_rad = float(tokens[20])
    zoom = float(tokens[21])
    yaw_rad = float(tokens[22])
    pitch_rad = float(tokens[23])
    roll_rad = float(tokens[24])

    emit_camera = tokens[25] == "true"
    num_subjects = int(tokens[26])
    if len(tokens) != 27 + num_subjects:
        raise SystemExit(
            f"Expected {27 + num_subjects} args, got {len(tokens)}"
        )
    subject_object_names = list(tokens[27 : 27 + num_subjects])

    return (
        input_fbx,
        output_png,
        resolution_x,
        resolution_y,
        projection,
        lens_mode,
        lens_value,
        ortho_scale,
        near,
        far,
        renderer,
        samples,
        lighting,
        explicit_pose,
        orbit_h_rad,
        orbit_v_rad,
        zoom,
        yaw_rad,
        pitch_rad,
        roll_rad,
        emit_camera,
        subject_object_names,
    )


def add_camera(cam_pos, cam_quat, projection, lens_mode, lens_value, ortho_scale, near, far):
    cam_data = bpy.data.cameras.new("RenderCamera")
    cam_data.type = projection
    cam_data.clip_start = near
    cam_data.clip_end = far

    if projection == "PERSP":
        if lens_mode == "fov":
            cam_data.lens_unit = "FOV"
            cam_data.angle = math.radians(lens_value)
        else:
            cam_data.lens_unit = "MILLIMETERS"
            cam_data.lens = lens_value
    else:
        if ortho_scale > 0.0:
            cam_data.ortho_scale = ortho_scale
        else:
            min_c, max_c = scene_bounds()
            diagonal = (max_c - min_c).length
            cam_data.ortho_scale = max(diagonal, 1.0)

    cam_obj = bpy.data.objects.new("RenderCamera", cam_data)
    bpy.context.scene.collection.objects.link(cam_obj)
    cam_obj.location = cam_pos
    cam_obj.rotation_mode = "QUATERNION"
    cam_obj.rotation_quaternion = cam_quat
    bpy.context.scene.camera = cam_obj
    return cam_obj


def _camera_basis(cam_forward):
    forward = cam_forward.copy()
    if forward.length == 0.0:
        forward = Vector((0.0, 1.0, 0.0))
    forward.normalize()
    world_up = Vector((0.0, 0.0, 1.0))
    right = forward.cross(world_up)
    if right.length < 1.0e-6:
        right = Vector((1.0, 0.0, 0.0))
    right.normalize()
    up = right.cross(forward).normalized()
    return forward, right, up, world_up


def add_three_point_lights(cam_forward, energy_scale):
    """Key / fill / rim area lights positioned relative to the scene bounds.
    `energy_scale` multiplies the per-light wattage so callers can pick a
    brighter or softer variant."""
    min_c, max_c = scene_bounds()
    center = (min_c + max_c) * 0.5
    diagonal = (max_c - min_c).length
    distance = max(diagonal * 1.5, 2.0)

    forward, right, up, world_up = _camera_basis(cam_forward)

    def add_light(name, offset, energy):
        data = bpy.data.lights.new(name=name, type="AREA")
        data.energy = energy * energy_scale
        data.size = max(diagonal, 1.0)
        obj = bpy.data.objects.new(name=name, object_data=data)
        obj.location = center + offset
        direction = (center - obj.location).normalized()
        obj.rotation_mode = "QUATERNION"
        obj.rotation_quaternion = look_at_quaternion(
            obj.location,
            center,
            world_up if abs(direction.z) < 0.99 else Vector((0.0, 1.0, 0.0)),
        )
        bpy.context.scene.collection.objects.link(obj)

    key = (right * 0.7 + up * 0.6 - forward * 0.8).normalized() * distance
    fill = (-right * 0.8 + up * 0.2 - forward * 0.3).normalized() * distance
    rim = (forward * 0.6 + up * 0.8).normalized() * distance

    add_light("KeyLight", key, 1500.0)
    add_light("FillLight", fill, 600.0)
    add_light("RimLight", rim, 900.0)


def add_flat_light(cam_forward):
    """Single camera-aligned sun for even, low-contrast illumination."""
    data = bpy.data.lights.new(name="FlatSun", type="SUN")
    data.energy = 2.0
    obj = bpy.data.objects.new(name="FlatSun", object_data=data)
    bpy.context.scene.collection.objects.link(obj)
    forward, _, _, world_up = _camera_basis(cam_forward)
    sun_origin = -forward
    target = Vector((0.0, 0.0, 0.0))
    obj.location = sun_origin
    obj.rotation_mode = "QUATERNION"
    obj.rotation_quaternion = look_at_quaternion(
        sun_origin,
        target,
        world_up if abs(forward.z) < 0.99 else Vector((0.0, 1.0, 0.0)),
    )


def set_world_strength(strength):
    """Sets the world background to a uniform white with the given strength,
    creating the world / nodes / shader graph if needed. `World.use_nodes` is
    deprecated in Blender 5.0 and issues a DeprecationWarning on write, so it
    is only set when nodes are not already enabled, and the warning is
    suppressed to keep stderr clean for the caller."""
    world = bpy.context.scene.world
    if world is None:
        world = bpy.data.worlds.new("World")
        bpy.context.scene.world = world
    if world.node_tree is None:
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", DeprecationWarning)
            world.use_nodes = True
    tree = world.node_tree
    bg = tree.nodes.get("Background")
    if bg is None:
        bg = tree.nodes.new("ShaderNodeBackground")
    output = tree.nodes.get("World Output")
    if output is None:
        output = tree.nodes.new("ShaderNodeOutputWorld")
    if not any(
        link.from_node is bg and link.to_node is output for link in tree.links
    ):
        tree.links.new(bg.outputs[0], output.inputs["Surface"])
    bg.inputs[0].default_value = (1.0, 1.0, 1.0, 1.0)
    bg.inputs[1].default_value = strength


def apply_lighting(lighting, cam_forward):
    if lighting == "environment":
        set_world_strength(1.5)
    elif lighting == "three-point":
        set_world_strength(0.0)
        add_three_point_lights(cam_forward, energy_scale=1.0)
    elif lighting == "studio":
        set_world_strength(0.0)
        add_three_point_lights(cam_forward, energy_scale=0.33)
    elif lighting == "flat":
        set_world_strength(0.0)
        add_flat_light(cam_forward)
    elif lighting == "none":
        pass
    else:
        raise SystemExit(f"Unknown lighting mode: {lighting}")


def configure_render(renderer, resolution_x, resolution_y, samples, output_png):
    scene = bpy.context.scene
    scene.render.engine = renderer
    scene.render.resolution_x = resolution_x
    scene.render.resolution_y = resolution_y
    scene.render.resolution_percentage = 100
    scene.render.film_transparent = False
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.render.filepath = output_png

    if renderer == "CYCLES":
        scene.cycles.samples = samples
        scene.cycles.use_denoising = True
    else:
        if hasattr(scene, "eevee"):
            eevee = scene.eevee
            if hasattr(eevee, "taa_render_samples"):
                eevee.taa_render_samples = samples


def emit_camera_block(cam_pos, cam_quat):
    euler = cam_quat.to_euler("XYZ")
    print("===CAMERA===")
    print(f"POSITION {cam_pos.x} {cam_pos.y} {cam_pos.z}")
    print(f"ROTATION {euler.x} {euler.y} {euler.z}")
    print("===/CAMERA===")


def main():
    (
        input_fbx,
        output_png,
        resolution_x,
        resolution_y,
        projection,
        lens_mode,
        lens_value,
        ortho_scale,
        near,
        far,
        renderer,
        samples,
        lighting,
        explicit_pose,
        orbit_h_rad,
        orbit_v_rad,
        zoom,
        yaw_rad,
        pitch_rad,
        roll_rad,
        emit_camera,
        subject_object_names,
    ) = parse_args()

    reset_scene()
    import_fbx(input_fbx)

    cam_pos, cam_quat = resolve_camera(
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
    )

    add_camera(cam_pos, cam_quat, projection, lens_mode, lens_value, ortho_scale, near, far)
    cam_forward = cam_quat @ Vector((0.0, 0.0, -1.0))
    apply_lighting(lighting, cam_forward)
    configure_render(renderer, resolution_x, resolution_y, samples, output_png)

    bpy.ops.render.render(write_still=True)

    if emit_camera:
        emit_camera_block(cam_pos, cam_quat)


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
