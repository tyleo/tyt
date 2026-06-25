use crate::{
    GoxlBlock, GoxlCamera, GoxlImage, GoxlLayer, GoxlLight, GoxlMaterial, GoxlPreview,
    GoxlUnknownChunk,
};

/// A parsed Goxel `.gox` file: every chunk under the `"GOX "` header, decoded
/// into typed fields.
#[derive(Clone, Debug, PartialEq)]
pub struct GoxlFile {
    /// Format version from the header; the reference exporter writes `2`.
    pub version: i32,

    /// The `IMG ` image metadata.
    pub image: GoxlImage,

    /// The `PREV` preview thumbnail, or `None` when the file omits it.
    pub preview: Option<GoxlPreview>,

    /// The shared `BL16` voxel blocks, in stored order; a layer references one
    /// by index.
    pub blocks: Vec<GoxlBlock>,

    /// The `MATE` materials, in stored order; a layer references one by index.
    pub materials: Vec<GoxlMaterial>,

    /// The `LAYR` layers, in stored order.
    pub layers: Vec<GoxlLayer>,

    /// The `CAMR` cameras, in stored order.
    pub cameras: Vec<GoxlCamera>,

    /// The `LIGH` light settings, or `None` when the file omits them.
    pub light: Option<GoxlLight>,

    /// Chunks this crate does not model, preserved verbatim.
    pub unknown_chunks: Vec<GoxlUnknownChunk>,
}

impl Default for GoxlFile {
    fn default() -> Self {
        Self {
            version: 2,
            image: GoxlImage::default(),
            preview: None,
            blocks: Vec::new(),
            materials: Vec::new(),
            layers: Vec::new(),
            cameras: Vec::new(),
            light: None,
            unknown_chunks: Vec::new(),
        }
    }
}
