use std::num::NonZeroU32;
use wgpu::{
    Color, CommandEncoder, LoadOp, Operations, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp, TextureView,
};

/// The two-layer textures a frame of a [`BoxView`](crate::BoxView) draws
/// to, a layer an eye.
#[derive(Clone, Copy)]
pub struct ViewTarget<'a> {
    pub color: &'a TextureView,

    /// The depth, in [`BoxView::DEPTH`](crate::BoxView::DEPTH).
    pub depth: &'a TextureView,

    /// Whether the depth outlasts the frame. A tiled GPU saves a store
    /// where it does not.
    pub keep_depth: bool,
}

impl ViewTarget<'_> {
    /// The mask of a multiview pass over both eyes.
    pub(crate) const EYES: Option<NonZeroU32> = NonZeroU32::new(0b11);

    /// Begins one of a frame's passes over both eyes. With `clear`, the
    /// pass clears the color to it and the depth to zero, where any other
    /// loads what the pass before stored. The frame's `last` pass drops the
    /// depth unless the target keeps it.
    pub(crate) fn begin<'e>(
        &self,
        encoder: &'e mut CommandEncoder,
        label: &str,
        clear: Option<Color>,
        last: bool,
    ) -> RenderPass<'e> {
        let store = if last && !self.keep_depth {
            StoreOp::Discard
        } else {
            StoreOp::Store
        };

        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: self.color,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: clear.map_or(LoadOp::Load, LoadOp::Clear),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: self.depth,
                depth_ops: Some(Operations {
                    load: clear.map_or(LoadOp::Load, |_| LoadOp::Clear(0.0)),
                    store,
                }),
                stencil_ops: None,
            }),
            multiview_mask: Self::EYES,
            ..RenderPassDescriptor::default()
        })
    }
}
