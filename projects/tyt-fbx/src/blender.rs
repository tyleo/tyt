use crate::Script;

macro_rules! embed_blender_script {
    ($rel_path:literal) => {
        Script {
            relative_file_path: $rel_path,
            content: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/blender/",
                $rel_path,
            )),
        }
    };
}

pub const COMMON_PY: Script = embed_blender_script!("common.py");
pub const FBX_EXTRACT_MESH_PY: Script = embed_blender_script!("fbx_extract_mesh.py");
pub const FBX_HIERARCHY_PY: Script = embed_blender_script!("fbx_hierarchy.py");
pub const FBX_REDUCE_TO_SINGLE_MESH_PY: Script =
    embed_blender_script!("fbx_reduce_to_single_mesh.py");
pub const EXTRACT_FACES_AND_VERTICES_PY: Script =
    embed_blender_script!("extract_faces_and_vertices.py");
pub const FBX_RENAME_MESHES_PY: Script = embed_blender_script!("fbx_rename_meshes.py");
