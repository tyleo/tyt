import bpy
import math
import sys

from mathutils import Vector

from common import reset_scene


def parse_args():
    argv = sys.argv
    if "--" not in argv:
        raise SystemExit(
            "Usage: blender -b --python script.py -- <input_fbx> <output_png> "
            "<cam_pos_x> <cam_pos_y> <cam_pos_z> "
            "<cam_target_x> <cam_target_y> <cam_target_z> "
            "<cam_up_x> <cam_up_y> <cam_up_z> "
            "<resolution_x> <resolution_y> "
            "<focal_length> <renderer> <samples>"
        )

    tokens = argv[argv.index("--") + 1 :]
    if len(tokens) != 17:
        raise SystemExit(f"Expected 17 args, got {len(tokens)}")

    input_fbx = tokens[0]
    output_png = tokens[1]
    cam_pos = Vector((float(tokens[2]), float(tokens[3]), float(tokens[4])))
    cam_target = Vector((float(tokens[5]), float(tokens[6]), float(tokens[7])))
    cam_up = Vector((float(tokens[8]), float(tokens[9]), float(tokens[10])))
    resolution_x = int(tokens[11])
    resolution_y = int(tokens[12])
    focal_length = float(tokens[13])
    renderer = tokens[14]
    samples = int(tokens[15])
    auto_frame = tokens[16] == "1"
    return (
        input_fbx, output_png,
        cam_pos, cam_target, cam_up,
        resolution_x, resolution_y,
        focal_length, renderer, samples,
        auto_frame,
    )


def scene_bounds():
    """Returns (min_corner, max_corner) over all mesh vertices in world space.
    Falls back to a unit box around the origin when no meshes are present."""
    min_corner = Vector((math.inf, math.inf, math.inf))
    max_corner = Vector((-math.inf, -math.inf, -math.inf))
    found = False
    for obj in bpy.data.objects:
        if obj.type != "MESH":
            continue
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
    # Blender cameras look down -Z, so forward is -Z.
    z_axis = -forward
    x_axis = up.cross(z_axis).normalized()
    y_axis = z_axis.cross(x_axis).normalized()
    from mathutils import Matrix
    rot = Matrix((
        (x_axis.x, y_axis.x, z_axis.x, 0.0),
        (x_axis.y, y_axis.y, z_axis.y, 0.0),
        (x_axis.z, y_axis.z, z_axis.z, 0.0),
        (0.0, 0.0, 0.0, 1.0),
    ))
    return rot.to_quaternion()


def add_camera(cam_pos, cam_target, cam_up, focal_length):
    cam_data = bpy.data.cameras.new("RenderCamera")
    cam_data.lens = focal_length
    cam_obj = bpy.data.objects.new("RenderCamera", cam_data)
    bpy.context.scene.collection.objects.link(cam_obj)
    cam_obj.location = cam_pos
    cam_obj.rotation_mode = "QUATERNION"
    cam_obj.rotation_quaternion = look_at_quaternion(cam_pos, cam_target, cam_up)
    bpy.context.scene.camera = cam_obj
    return cam_obj


def add_three_point_lights(cam_pos, cam_target):
    """Adds key / fill / rim lights positioned relative to the scene bounds so
    meshes without materials still read clearly."""
    min_c, max_c = scene_bounds()
    center = (min_c + max_c) * 0.5
    diagonal = (max_c - min_c).length
    distance = max(diagonal * 1.5, 2.0)

    forward = (cam_target - cam_pos)
    if forward.length == 0.0:
        forward = Vector((0.0, 1.0, 0.0))
    forward.normalize()
    world_up = Vector((0.0, 0.0, 1.0))
    right = forward.cross(world_up)
    if right.length < 1.0e-6:
        right = Vector((1.0, 0.0, 0.0))
    right.normalize()
    up = right.cross(forward).normalized()

    def add_light(name, offset, energy, light_type="AREA"):
        data = bpy.data.lights.new(name=name, type=light_type)
        data.energy = energy
        if light_type == "AREA":
            data.size = max(diagonal, 1.0)
        obj = bpy.data.objects.new(name=name, object_data=data)
        obj.location = center + offset
        direction = (center - obj.location).normalized()
        obj.rotation_mode = "QUATERNION"
        obj.rotation_quaternion = look_at_quaternion(obj.location, center, world_up if abs(direction.z) < 0.99 else Vector((0.0, 1.0, 0.0)))
        bpy.context.scene.collection.objects.link(obj)

    key = (right * 0.7 + up * 0.6 - forward * 0.8).normalized() * distance
    fill = (-right * 0.8 + up * 0.2 - forward * 0.3).normalized() * distance
    rim = (forward * 0.6 + up * 0.8).normalized() * distance

    add_light("KeyLight", key, 1500.0)
    add_light("FillLight", fill, 600.0)
    add_light("RimLight", rim, 900.0)


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
        # Eevee / Eevee-Next.
        if hasattr(scene, "eevee"):
            eevee = scene.eevee
            if hasattr(eevee, "taa_render_samples"):
                eevee.taa_render_samples = samples


def auto_frame_camera(focal_length, resolution_x, resolution_y):
    """Computes a camera position and target that frames the scene bounds."""
    min_c, max_c = scene_bounds()
    center = (min_c + max_c) * 0.5
    diagonal = (max_c - min_c).length
    # Camera sensor width defaults to 36mm in Blender.
    sensor_width = 36.0
    aspect = max(resolution_x, 1) / max(resolution_y, 1)
    fit_size = diagonal / max(min(1.0, aspect), 1.0e-6)
    # Distance such that `fit_size` roughly fits horizontally in frame.
    distance = max((fit_size * focal_length) / sensor_width, diagonal * 1.5, 2.0)
    direction = Vector((1.0, -1.0, 0.7)).normalized()
    cam_pos = center + direction * distance
    cam_target = center
    cam_up = Vector((0.0, 0.0, 1.0))
    return cam_pos, cam_target, cam_up


def main():
    (
        input_fbx, output_png,
        cam_pos, cam_target, cam_up,
        resolution_x, resolution_y,
        focal_length, renderer, samples,
        auto_frame,
    ) = parse_args()

    reset_scene()
    bpy.ops.import_scene.fbx(filepath=input_fbx)

    if auto_frame:
        cam_pos, cam_target, cam_up = auto_frame_camera(
            focal_length, resolution_x, resolution_y
        )

    add_camera(cam_pos, cam_target, cam_up, focal_length)
    add_three_point_lights(cam_pos, cam_target)
    configure_render(renderer, resolution_x, resolution_y, samples, output_png)

    bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
