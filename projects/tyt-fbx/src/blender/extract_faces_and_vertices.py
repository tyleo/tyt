import bmesh
import bpy
import json
import sys


def parse_args():
    argv = sys.argv
    if "--" not in argv:
        raise SystemExit(
            "Usage: blender -b --python script.py -- <input_fbx> <mesh_name>"
        )

    tokens = argv[argv.index("--") + 1 :]
    if len(tokens) != 2:
        raise SystemExit("Expected 2 args: <input_fbx> <mesh_name>")

    return tokens[0], tokens[1]


def main():
    input_fbx, mesh_name = parse_args()

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.fbx(filepath=input_fbx)

    obj = bpy.data.objects.get(mesh_name)
    if obj is None or obj.type != "MESH":
        raise RuntimeError(f"Mesh object '{mesh_name}' not found")

    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bmesh.ops.triangulate(bm, faces=bm.faces[:])

    vertices = [{"x": v.co.x, "y": v.co.y, "z": v.co.z} for v in bm.verts]
    triangles = [[v.index for v in f.verts] for f in bm.faces]

    bm.free()

    print(json.dumps({"vertices": vertices, "triangles": triangles}))


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
