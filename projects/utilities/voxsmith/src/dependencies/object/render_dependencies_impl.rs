use crate::dependencies::DependenciesImpl;
use branded_id::U32Id;
use voxrender::{
    BRenderView, Render, RenderBloom, RenderImage, RenderOcclusion, RenderScene, Result,
};

impl Render for DependenciesImpl {
    fn render(
        &self,
        scene: &RenderScene,
        view_id: U32Id<BRenderView>,
        occlusion: RenderOcclusion,
        bloom: RenderBloom,
        width: u32,
        height: u32,
    ) -> Result<RenderImage> {
        voxrender_cpu::render(scene, view_id, occlusion, bloom, width, height)
    }
}
