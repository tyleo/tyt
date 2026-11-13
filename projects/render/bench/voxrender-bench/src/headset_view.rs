use crate::{BoxView, HeadsetEye, HeadsetStand, TileScene, ViewEye};
use ty_math::{TyQuaternionF64, TyVector3F64};
use voxrender::RenderBloom;
use wgpu::{Color, CommandEncoder, Device, Queue, RenderPass, TextureFormat};

/// A [`BoxView`] of a scene around a headset's wearer.
pub struct HeadsetView {
    boxes: BoxView,

    /// Where the headset's tracking space sits in the scene.
    pub stand: HeadsetStand,
}

impl HeadsetView {
    /// The format of the depth a pass that draws the view clears to zero.
    pub const DEPTH: TextureFormat = BoxView::DEPTH;

    /// Builds the view of `tiles` under `bloom` from `stand` for a two-layer
    /// target of `format` whose width and height are `size`.
    pub fn new(
        device: &Device,
        queue: &Queue,
        tiles: &TileScene,
        bloom: RenderBloom,
        stand: HeadsetStand,
        format: TextureFormat,
        size: [u32; 2],
    ) -> Self {
        HeadsetView {
            boxes: BoxView::new(device, queue, tiles, bloom, format, size),
            stand,
        }
    }

    /// The color a pass that draws the view clears its target to.
    pub fn sky(&self) -> Color {
        self.boxes.sky()
    }

    /// Readies the next draw for `eyes`, the headset's left eye and then
    /// its right.
    pub fn prepare(&mut self, queue: &Queue, eyes: &[HeadsetEye; 2]) {
        let turn = self.stand.rotation();

        let eyes = eyes.map(|eye| {
            let [x, y, z] = eye.position.map(f64::from);
            let [left, right, up, down] = eye.tangents.map(f64::from);
            let rotation = TyQuaternionF64::from_array(eye.rotation.map(f64::from));

            ViewEye::new(
                self.stand.to_scene(TyVector3F64::new(x, y, z)),
                turn * rotation,
                left,
                right,
                up,
                down,
            )
        });

        self.boxes.prepare(queue, &eyes, self.stand.scale);
    }

    /// How many passes draw the view, as [`BoxView::passes`] counts them.
    pub fn passes(&self) -> u32 {
        self.boxes.passes()
    }

    /// Draws the `at`th of the view's [`passes`](Self::passes) into `pass`.
    pub fn draw(&self, pass: &mut RenderPass<'_>, at: u32) {
        self.boxes.draw(pass, at);
    }

    /// Encodes the passes that come before the first of a frame's
    /// [`passes`](Self::passes), as [`BoxView::glow`] does.
    pub fn glow(&self, encoder: &mut CommandEncoder) {
        self.boxes.glow(encoder);
    }
}
