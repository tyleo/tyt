use crate::{BoxView, Error, HeadsetEye, Result, ViewEye, bench_scene};
use ty_math::{TyQuaternionF64, TyVector3F64};
use wgpu::{Color, Device, Queue, RenderPass, TextureFormat};

/// A [`BoxView`] of a bench scene around a headset's wearer. The origin of
/// the headset's tracking space stands at the scene's first view and faces
/// the way that view does, kept level.
pub struct HeadsetView {
    boxes: BoxView,

    /// The tracking space's origin in the scene.
    origin: TyVector3F64,

    /// The turn about +Y from the tracking space to the scene.
    turn: TyQuaternionF64,
}

impl HeadsetView {
    /// The format of the depth a pass that draws the view clears to zero.
    pub const DEPTH: TextureFormat = BoxView::DEPTH;

    /// Builds the view of the bench scene whose name starts with `scene`,
    /// for a two-layer target of `format`. Errors unless a scene has the
    /// name and a view.
    pub fn new(device: &Device, scene: &str, format: TextureFormat) -> Result<Self> {
        let render_scene = bench_scene(scene).ok_or_else(|| Error::NoScene(scene.to_owned()))?;
        let (_, view) = render_scene.iter_views().next().ok_or(Error::NoView)?;
        let forward = view.pose.rotation * -TyVector3F64::Z;

        Ok(HeadsetView {
            boxes: BoxView::new(device, &render_scene, format)?,
            origin: view.pose.position,
            turn: TyQuaternionF64::from_rotation_y((-forward.x).atan2(-forward.z)),
        })
    }

    /// The color a pass that draws the view clears its target to.
    pub fn sky(&self) -> Color {
        self.boxes.sky()
    }

    /// Readies the next draw for `eyes`, the headset's left eye and then
    /// its right.
    pub fn prepare(&mut self, queue: &Queue, eyes: &[HeadsetEye; 2]) {
        let eyes = eyes.map(|eye| {
            let [x, y, z] = eye.position.map(f64::from);
            let [left, right, up, down] = eye.tangents.map(f64::from);
            let rotation = TyQuaternionF64::from_array(eye.rotation.map(f64::from));

            ViewEye::new(
                self.origin + self.turn * TyVector3F64::new(x, y, z),
                self.turn * rotation,
                left,
                right,
                up,
                down,
            )
        });

        self.boxes.prepare(queue, &eyes);
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
