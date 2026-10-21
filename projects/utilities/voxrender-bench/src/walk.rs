use crate::{Error, Result, Tree64};
use branded_id::U32Id;
use voxrender::{BRenderView, RenderScene};
use voxrender_wgpu::{HitTrace, dense_cells};
use wgpu::Device;

/// The traversal a benchmark traces.
#[derive(Clone, Copy, Debug)]
pub enum Walk {
    /// `voxrender-wgpu`'s integer walk.
    Integer,

    /// An f32 DDA over the integer walk's inputs, for the record.
    Float,

    /// The integer walk over each object's [`Tree64`], crossing a region of
    /// one value in a single jump.
    Tree,

    /// The tree walk, with each pixel's hit reporting the loop iterations of
    /// its walks over every placement as its voxel id. The scene must have
    /// no shadowed lights.
    CountedTree,
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
            Walk::Integer => {
                HitTrace::new(device, scene, view_id, width, height).map_err(Error::Trace)
            }

            Walk::Float => HitTrace::with_walk(
                device,
                scene,
                view_id,
                width,
                height,
                include_str!("float_walk.wgsl"),
                dense_cells,
            )
            .map_err(Error::Trace),

            Walk::Tree => trace_tree(
                device,
                scene,
                view_id,
                width,
                height,
                include_str!("plain_tree_walk.wgsl"),
            ),

            Walk::CountedTree => trace_tree(
                device,
                scene,
                view_id,
                width,
                height,
                include_str!("counted_tree_walk.wgsl"),
            ),
        }
    }
}

/// The tree walk's trace of `view_id`. `walk` is the WGSL that defines
/// `walk` over `tree_walk.wgsl`.
fn trace_tree(
    device: &Device,
    scene: &RenderScene,
    view_id: U32Id<BRenderView>,
    width: u32,
    height: u32,
    walk: &str,
) -> Result<HitTrace> {
    for (_, object) in scene.iter_objects() {
        Tree64::new(object)?;
    }

    HitTrace::with_walk(
        device,
        scene,
        view_id,
        width,
        height,
        &format!("{}\n{walk}", include_str!("tree_walk.wgsl")),
        |object| {
            Tree64::new(object)
                .expect("every object's tree built above")
                .words()
                .to_vec()
        },
    )
    .map_err(Error::Trace)
}
