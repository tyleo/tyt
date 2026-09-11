use crate::{
    Result, VoxDocumentFile, box_ext,
    ext::{VoxconvVoxMain, composite_vox_ext_from_voxj_vox_ext},
    single_file_bytes,
};
use voxj::dependencies::DecodeBase64;
use voxj_voxcore::codec::{
    dependencies::{DecodeVoxjJson, Inflate},
    from_voxj_bytes_with_ext,
};

/// Decodes a `.voxj` or `.voxjz` document into a state carrying its `ext`
/// block as a [`CompositeVoxExt`](crate::ext::CompositeVoxExt), with each
/// enabled format's entry as that format's ext.
pub fn read_voxj_with_ext<D: DecodeBase64 + DecodeVoxjJson + Inflate>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxconvVoxMain> {
    let (state, ext) =
        from_voxj_bytes_with_ext(dependencies, single_file_bytes(files)?)?.take_ext();
    Ok(box_ext(
        state.put_ext(composite_vox_ext_from_voxj_vox_ext(ext)?),
    ))
}
