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

pub const FBX_EXTRACT_MESH_PY: Script = embed_blender_script!("fbx_extract_mesh.py");
