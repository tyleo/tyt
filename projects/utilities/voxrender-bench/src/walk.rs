use crate::{Error, Result};
use branded_id::U32Id;
use voxrender::{BRenderView, RenderScene};
use voxrender_wgpu::HitTrace;
use wgpu::Device;

/// The traversal a benchmark traces.
#[derive(Clone, Copy, Debug)]
pub enum Walk {
    /// `voxrender-wgpu`'s integer walk.
    Integer,

    /// An f32 DDA over the integer walk's inputs, for the record.
    Float,
}

impl Walk {
    /// The walk's trace of `view_id` over a `width` by `height` image.
    pub fn trace(
        self,
        device: &Device,
        scene: &RenderScene,
        view_id: U32Id<BRenderView>,
        width: u32,
        height: u32,
    ) -> Result<HitTrace> {
        match self {
            Walk::Integer => HitTrace::new(device, scene, view_id, width, height),

            Walk::Float => HitTrace::with_walk(
                device,
                scene,
                view_id,
                width,
                height,
                include_str!("float_walk.wgsl"),
            ),
        }
        .map_err(Error::Trace)
    }
}
