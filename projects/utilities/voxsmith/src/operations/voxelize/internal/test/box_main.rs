use crate::operations::voxelize::{box_primitive, document_of};
use meshdoc::{MeshMain, MeshMaterial};
use ty_math::{TyLinSrgbaF64, TyTransformF64};

/// A document of one box spanning `[0, sx]`, `[0, sy]`, `[0, sz]` under a
/// node named `node_name`. The primitive draws `material` as `(base_color,
/// metallic, roughness)`, or the default when `None`.
pub(crate) fn box_main(
    sx: f64,
    sy: f64,
    sz: f64,
    material: Option<([f64; 4], f64, f64)>,
    node_name: Option<&str>,
) -> MeshMain<()> {
    let mut main = MeshMain::default();

    let material_id = material.map(|(color, metallic, roughness)| {
        main.retain_material(MeshMaterial {
            base_color_factor: TyLinSrgbaF64::from(color),
            metallic_factor: metallic,
            roughness_factor: roughness,
            ..Default::default()
        })
        .unwrap()
    });

    let mut primitive = box_primitive(sx, sy, sz);
    primitive.set_material_id(material_id);

    document_of(main, primitive, node_name, TyTransformF64::default())
}
