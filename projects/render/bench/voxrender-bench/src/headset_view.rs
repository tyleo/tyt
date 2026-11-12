use crate::{BoxView, HeadsetEye, HeadsetStand, Result, ViewEye};
use ty_math::{TyQuaternionF64, TyVector3F64};
use voxrender::RenderScene;
use wgpu::{Color, Device, Queue, RenderPass, TextureFormat};

/// A [`BoxView`] of a scene around a headset's wearer.
pub struct HeadsetView {
    boxes: BoxView,

    /// Where the headset's tracking space sits in the scene.
    pub stand: HeadsetStand,
}

impl HeadsetView {
    /// The format of the depth a pass that draws the view clears to zero.
    pub const DEPTH: TextureFormat = BoxView::DEPTH;

    /// Builds the view of `scene` from `stand`, for a two-layer target of
    /// `format` and of `size`, its width and height. Errors as
    /// [`BoxView::new`] does.
    pub fn new(
        device: &Device,
        queue: &Queue,
        scene: &RenderScene,
        stand: HeadsetStand,
        format: TextureFormat,
        size: [u32; 2],
    ) -> Result<Self> {
        Ok(HeadsetView {
            boxes: BoxView::new(device, queue, scene, format, size)?,
            stand,
        })
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
}
