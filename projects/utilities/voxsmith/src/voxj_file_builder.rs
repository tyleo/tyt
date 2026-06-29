use crate::{EditStateMode, Result, write_voxj};
use voxcore::VoxMain;
use voxj::VoxjFile;
use voxj_codec::{PositionEncoding, SampleEncoding};

/// Builds a [`VoxjFile`] from a [`VoxMain`], the configurable form of
/// [`to_voxj_file`](crate::to_voxj_file). It defaults to the smallest per-object
/// block encodings, keeps the user-defined ext block, and records the edit state
/// automatically, the same document that function writes.
pub struct VoxjFileBuilder<'a> {
    state: &'a VoxMain,
    encoding: Option<(PositionEncoding, SampleEncoding)>,
    ext: bool,
    edit_state: EditStateMode,
}

impl<'a> VoxjFileBuilder<'a> {
    /// Starts a builder encoding `state` into a Voxel Json document.
    pub fn new(state: &'a VoxMain) -> Self {
        Self {
            state,
            encoding: None,
            ext: true,
            edit_state: EditStateMode::Auto,
        }
    }

    /// Sets the block encoding applied to every object: a fixed position/sample
    /// pair, or `None` to search for the smallest per-object encoding. Defaults
    /// to `None`.
    pub fn encoding(mut self, encoding: Option<(PositionEncoding, SampleEncoding)>) -> Self {
        self.encoding = encoding;
        self
    }

    /// Keeps (the default) or drops the user-defined `ext` extension block.
    pub fn ext(mut self, ext: bool) -> Self {
        self.ext = ext;
        self
    }

    /// Sets when each object's editor build volume is recorded in the document's
    /// edit state.
    pub fn edit_state(mut self, edit_state: EditStateMode) -> Self {
        self.edit_state = edit_state;
        self
    }

    /// Builds the [`VoxjFile`].
    pub fn build(self) -> Result<VoxjFile> {
        write_voxj(self.state, self.encoding, self.ext, self.edit_state)
    }
}
