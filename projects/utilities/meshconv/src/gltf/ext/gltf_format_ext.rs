use crate::{
    Dependencies, MeshDocumentFile, Result,
    ext::{FormatExt, MeshconvMeshMain, box_ext},
    gltf::{Gltf, GltfContainer, GltfWriteFormat, document_files, loose_files},
};
use gltf_meshdoc::{
    GltfExt,
    codec::{from_gltf_bytes, to_glb_bytes, to_gltf_bytes},
    to_gltf_mesh_main,
};

impl FormatExt for Gltf {
    fn read_with_ext<D: Dependencies>(
        dependencies: &D,
        files: &[MeshDocumentFile],
    ) -> Result<MeshconvMeshMain> {
        Ok(box_ext(from_gltf_bytes(
            dependencies.gltf(),
            MeshDocumentFile::primary_bytes(files)?,
            loose_files(files),
        )?))
    }

    fn write_with_ext<D: Dependencies>(
        dependencies: &D,
        options: &GltfWriteFormat,
        main: MeshconvMeshMain,
    ) -> Result<Vec<MeshDocumentFile>> {
        let main = match main.ext().downcast_ref::<GltfExt>() {
            Some(ext) => {
                let ext = ext.clone();
                main.take_ext().main.put_ext(ext)
            }
            None => to_gltf_mesh_main(main.take_ext().main),
        };

        let bytes = match options.container {
            GltfContainer::Glb => to_glb_bytes(dependencies.gltf(), &main, &options.options)?,
            GltfContainer::Gltf => to_gltf_bytes(dependencies.gltf(), &main, &options.options)?,
        };

        Ok(document_files(bytes))
    }
}
