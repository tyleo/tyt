use crate::operations::voxelize::voxel_attribute;
use ty_math::{TyLinSrgbaF64, TyVector3U32};
use voxcore::{
    VoxMain, VoxValuePoolValueRef, color::srgba_u8_from_lin_srgba_f64, material::BASE_COLOR,
};

/// The `#RRGGBBAA` hex of the `baseColor` the voxel at `position` samples,
/// encoded to sRGB from the stored linear color.
pub(crate) fn voxel_hex(main: &VoxMain, position: TyVector3U32) -> String {
    let (value_pool, value_id) = voxel_attribute(main, position, BASE_COLOR);
    let VoxValuePoolValueRef::Vec4Float(components) = value_pool.value(value_id).unwrap() else {
        panic!("baseColor is a four-float color");
    };
    let bytes = <[u8; 4]>::from(srgba_u8_from_lin_srgba_f64(TyLinSrgbaF64::new(
        components[0],
        components[1],
        components[2],
        components[3],
    )));
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        bytes[0], bytes[1], bytes[2], bytes[3]
    )
}
