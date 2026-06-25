use crate::{
    MVoxCamera, MVoxLayer, MVoxMaterial, MVoxModel, MVoxPalette, MVoxRenderObject, MVoxSceneNode,
    MVoxUnknownChunk,
};

/// A parsed MagicaVoxel `.vox` file: every chunk under the root `MAIN`, decoded
/// into typed fields.
#[derive(Clone, Debug, PartialEq)]
pub struct MVoxFile {
    /// The format version from the header; the reference exporter writes `150`.
    pub version: u32,

    /// The models, in stored order; a shape node references one by index.
    pub models: Vec<MVoxModel>,

    /// The `RGBA` palette, or `None` when the file omits it and
    /// [`MVoxPalette::default`] applies.
    pub palette: Option<MVoxPalette>,

    /// The scene-graph nodes (`nTRN` / `nGRP` / `nSHP`), in stored order.
    pub scene_nodes: Vec<MVoxSceneNode>,

    /// The material definitions (`MATL`).
    pub materials: Vec<MVoxMaterial>,

    /// The layer definitions (`LAYR`).
    pub layers: Vec<MVoxLayer>,

    /// The render-settings chunks (`rOBJ`).
    pub render_objects: Vec<MVoxRenderObject>,

    /// The render cameras (`rCAM`).
    pub cameras: Vec<MVoxCamera>,

    /// The palette color names (`NOTE`), in stored order.
    pub palette_notes: Vec<String>,

    /// The palette index map (`IMAP`): 256 palette-index associations, or `None`
    /// when the file omits it.
    pub index_map: Option<[u8; 256]>,

    /// Chunks this crate does not model, preserved verbatim.
    pub unknown_chunks: Vec<MVoxUnknownChunk>,
}

impl MVoxFile {
    /// The file's palette, falling back to [`MVoxPalette::default`] (MagicaVoxel's
    /// built-in palette) when the file has no `RGBA` chunk.
    pub fn resolved_palette(&self) -> MVoxPalette {
        self.palette.clone().unwrap_or_default()
    }
}

impl Default for MVoxFile {
    fn default() -> Self {
        Self {
            version: 150,
            models: Vec::new(),
            palette: None,
            scene_nodes: Vec::new(),
            materials: Vec::new(),
            layers: Vec::new(),
            render_objects: Vec::new(),
            cameras: Vec::new(),
            palette_notes: Vec::new(),
            index_map: None,
            unknown_chunks: Vec::new(),
        }
    }
}
