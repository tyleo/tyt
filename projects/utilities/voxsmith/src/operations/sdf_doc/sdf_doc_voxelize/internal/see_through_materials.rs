use crate::operations::sdf_doc::SdfMaterialProperties;
use branded_id::{IdSlice, IdVec};
use sdfcore::BSdfMaterial;
use voxcore::material;

/// Whether a surface of each of `materials` lets any light through.
pub fn see_through_materials(
    materials: &IdSlice<BSdfMaterial, SdfMaterialProperties>,
) -> IdVec<BSdfMaterial, bool> {
    IdVec::from_vec(
        materials
            .iter()
            .map(|material| {
                let pass = material::pass(
                    material.base_color,
                    material.metallic,
                    material.transmission,
                    material.ior,
                );

                pass.red > 0.0 || pass.green > 0.0 || pass.blue > 0.0
            })
            .collect(),
    )
}
