use crate::{
    BoxDraw, BoxMarch, Error, HitPacking, HitTargets, RasterArm, Result, mask_boxes, tree_boxes,
};
use branded_id::U32Id;
use voxrender::{BRenderView, Error as RenderError, RenderScene};
use voxrender_wgpu::PixelHit;
use wgpu::{CommandEncoder, Device, QuerySet, Queue};

/// One pass draws each eye.
const PASSES: RasterArm = RasterArm {
    multiview: false,
    samples: 1,
};

/// A raster of one view over two `width` by `height` eyes. It draws the box
/// around each tile's live cells with a march in each fragment. Each pixel's
/// texel holds its first hit: the placement, the cell, and the
/// face. Both eyes draw the scene's view, as B2's traces do.
pub struct BoxRaster {
    boxes: BoxDraw,

    packing: HitPacking,

    targets: HitTargets,
}

impl BoxRaster {
    /// Builds the raster of `scene` through `view_id` over two `width` by
    /// `height` eyes with `march` in its fragments. Errors unless each hit
    /// packs into 32 bits.
    pub fn new(
        device: &Device,
        scene: &RenderScene,
        view_id: U32Id<BRenderView>,
        width: u32,
        height: u32,
        march: BoxMarch,
    ) -> Result<Self> {
        let targets = HitTargets::new(device, width, height, PASSES)?;
        let packing = HitPacking::new(scene)?;

        let view = scene
            .view(view_id)
            .ok_or(Error::Render(RenderError::UnknownView { view_id }))?;

        let boxes = match march {
            BoxMarch::Tree(march) => tree_boxes(
                device,
                scene,
                view,
                width,
                height,
                march,
                packing,
                targets.multiview_mask(),
            )?,

            BoxMarch::Masks { solids, depth } => mask_boxes(
                device,
                scene,
                view,
                width,
                height,
                solids,
                depth,
                packing,
                targets.multiview_mask(),
            )?,
        };

        Ok(BoxRaster {
            boxes,
            packing,
            targets,
        })
    }

    /// Records the raster of both eyes into `encoder`, between the
    /// timestamps at `first` and `first + 1` of `query_set`.
    pub fn record(&self, encoder: &mut CommandEncoder, query_set: &QuerySet, first: u32) {
        self.targets.draw(
            encoder,
            Some((query_set, first)),
            self.boxes.passes(),
            |pass, _, run| self.boxes.draw(pass, run),
        );
    }

    /// Draws once and reads back the first eye's hit through each pixel,
    /// row by row, with no shadow bits.
    pub fn trace(&self, device: &Device, queue: &Queue) -> Result<Vec<Option<PixelHit>>> {
        let words = self
            .targets
            .read(device, queue, self.boxes.passes(), |pass, _, run| {
                self.boxes.draw(pass, run)
            })?;

        Ok(words
            .into_iter()
            .map(|word| self.packing.unpack(word))
            .collect())
    }
}
