use branded_id::U32Id;
use ty_math::TyLinSrgbaF64;
use voxrender::{BRenderMaterial, RenderScene, fixtures::matte};

/// Adds an opaque matte material of linear `color` to `scene`.
pub fn retain_matte(scene: &mut RenderScene, [r, g, b]: [f64; 3]) -> U32Id<BRenderMaterial> {
    scene
        .retain_material(matte(TyLinSrgbaF64::new(r, g, b, 1.0)))
        .expect("a matte color is a valid material")
}
