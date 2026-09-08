use crate::{Result, VoxDocumentFile, single_file_bytes};
#[cfg(feature = "goxl")]
use goxl_voxcore::ext::GoxlExt;
#[cfg(feature = "mvox")]
use mvox_voxcore::ext::MVoxExt;
#[cfg(feature = "qbcl")]
use qbcl_voxcore::ext::{QbExt, QbclExt, QbtExt};
#[cfg(feature = "vmax")]
use vmax_voxcore::ext::VMaxExt;
use voxcore::{
    VoxMain, VoxMap,
    ext::{CompositeVoxExt, VoxExt},
};
use voxj::dependencies::DecodeBase64;
use voxj_voxcore::codec::{
    dependencies::{DecodeVoxjJson, Inflate},
    from_voxj_bytes,
};

/// Decodes a `.voxj` or `.voxjz` document into a state carrying its `ext`
/// block as a [`CompositeVoxExt`], with each enabled format's entry decoded
/// into that format's ext.
pub fn read_voxj_with_ext<D: DecodeBase64 + DecodeVoxjJson + Inflate>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<Box<dyn VoxExt>>> {
    let state: VoxMain<VoxMap> = from_voxj_bytes(dependencies, single_file_bytes(files)?)?;

    let composite = decode_entries(CompositeVoxExt::from(state.ext().clone()))?;

    Ok(state.map_ext(|_| Box::new(composite) as Box<dyn VoxExt>))
}

/// Decodes the entry of each enabled format.
fn decode_entries(composite: CompositeVoxExt) -> Result<CompositeVoxExt> {
    #[cfg(feature = "goxl")]
    let composite = composite.decode::<GoxlExt>()?;

    #[cfg(feature = "mvox")]
    let composite = composite.decode::<MVoxExt>()?;

    #[cfg(feature = "qbcl")]
    let composite = composite
        .decode::<QbExt>()?
        .decode::<QbtExt>()?
        .decode::<QbclExt>()?;

    #[cfg(feature = "vmax")]
    let composite = composite.decode::<VMaxExt>()?;

    Ok(composite)
}
