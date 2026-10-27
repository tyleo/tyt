use crate::{Error, Result, Tree64};
use branded_id::U32Id;
use voxrender::{BRenderView, RenderScene};
use voxrender_wgpu::{HitTrace, dense_cells};
use wgpu::{Device, ShaderRuntimeChecks};

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

    /// The tree walk with wgpu's runtime checks. Its time shows their cost.
    CheckedTree,

    /// The tree walk, with each pixel's hit reporting as its voxel id the
    /// loop iterations of its walks over every placement that leave a region
    /// of side `side`. The scene must have no shadowed lights.
    CountedTree {
        /// The side in cells of the regions whose iterations count.
        side: i32,
    },
}

impl Walk {
    /// The walk's trace of `view_id` over a `width` by `height` image. Every
    /// walk but [`Walk::CheckedTree`] runs without wgpu's runtime checks.
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

            // SAFETY: the float walk reads only its placements' cells, and
            // each of its loops ends within a grid's extent.
            Walk::Float => unsafe {
                HitTrace::with_walk(
                    device,
                    scene,
                    view_id,
                    width,
                    height,
                    include_str!("float_walk.wgsl"),
                    dense_cells,
                    ShaderRuntimeChecks::unchecked(),
                )
            }
            .map_err(Error::Trace),

            Walk::Tree => trace_tree(
                device,
                scene,
                view_id,
                width,
                height,
                include_str!("plain_tree_walk.wgsl"),
                ShaderRuntimeChecks::unchecked(),
            ),

            Walk::CheckedTree => trace_tree(
                device,
                scene,
                view_id,
                width,
                height,
                include_str!("plain_tree_walk.wgsl"),
                ShaderRuntimeChecks::checked(),
            ),

            Walk::CountedTree { side } => trace_tree(
                device,
                scene,
                view_id,
                width,
                height,
                &format!(
                    "const SIDE: i32 = {side};\n{}",
                    include_str!("counted_tree_walk.wgsl")
                ),
                ShaderRuntimeChecks::unchecked(),
            ),
        }
    }
}

/// The tree walk's trace of `view_id`. `walk` is the WGSL that defines
/// `walk` over `tree_walk.wgsl`, and the module keeps the runtime checks in
/// `checks`.
fn trace_tree(
    device: &Device,
    scene: &RenderScene,
    view_id: U32Id<BRenderView>,
    width: u32,
    height: u32,
    walk: &str,
    checks: ShaderRuntimeChecks,
) -> Result<HitTrace> {
    for (_, object) in scene.iter_objects() {
        Tree64::new(object)?;
    }

    // SAFETY: the tree walk reads only within its placements' trees, and each
    // of its loops ends within a grid's extent or a correction's window.
    unsafe {
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
            checks,
        )
    }
    .map_err(Error::Trace)
}
