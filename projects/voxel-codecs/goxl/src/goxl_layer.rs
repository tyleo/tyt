use crate::{GoxlDict, GoxlLayerBlock, GoxlShape};

/// A `LAYR` layer: a named, optionally hidden volume built either from placed
/// [`blocks`](Self::blocks), from a clone of another layer
/// ([`base_id`](Self::base_id)), or from a procedural [`shape`](Self::shape).
#[derive(Clone, Debug, PartialEq)]
pub struct GoxlLayer {
    /// Layer name.
    pub name: String,

    /// Unique id within the file, referenced by a cloning layer's
    /// [`base_id`](Self::base_id).
    pub id: i32,

    /// Id of the layer this one clones, or `0` when it is not a clone. A clone
    /// stores no blocks of its own.
    pub base_id: i32,

    /// Index into [`GoxlFile::materials`](crate::GoxlFile::materials) of the
    /// layer's material.
    pub material: i32,

    /// Volume blending mode (Goxel's `volume_utils.h` mode id).
    pub mode: i32,

    /// Whether the layer is visible.
    pub visible: bool,

    /// `4 x 4` transform applied to the layer, as 16 floats in stored order.
    pub transform: [[f32; 4]; 4],

    /// Placed blocks, in stored order. Empty for clone and shape layers.
    pub blocks: Vec<GoxlLayerBlock>,

    /// Optional `4 x 4` bounding box, present only when the layer sets one.
    pub bounding_box: Option<[[f32; 4]; 4]>,

    /// Path of the source image for a 2D image layer, if any.
    pub image_path: Option<String>,

    /// Procedural shape for a shape layer, if any.
    pub shape: Option<GoxlShape>,

    /// `[r, g, b, a]` color for a shape layer, if any.
    pub color: Option<[u8; 4]>,

    /// Dict keys this crate does not model, preserved verbatim.
    pub extra: GoxlDict,
}

impl Default for GoxlLayer {
    fn default() -> Self {
        Self {
            name: String::new(),
            id: 0,
            base_id: 0,
            material: 0,
            mode: 0,
            visible: true,
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            blocks: Vec::new(),
            bounding_box: None,
            image_path: None,
            shape: None,
            color: None,
            extra: GoxlDict::default(),
        }
    }
}
