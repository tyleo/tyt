use crate::{EditStateMode, Result, VoxjVoxMain, write_voxj};
use voxj::VoxjFile;
use voxj::objects::{PositionEncoding, SampleEncoding};

/// Builds a [`VoxjFile`] from a [`VoxjVoxMain`], the configurable form of
/// [`to_voxj_file`](crate::to_voxj_file). It defaults to the smallest
/// per-object block encodings, keeps the carried `ext` block, and records the
/// edit state automatically, reproducing the document that function writes.
pub struct VoxjFileBuilder<'a> {
    state: &'a VoxjVoxMain,
    position_encoding: Option<PositionEncoding>,
    sample_encoding: Option<SampleEncoding>,
    ext: bool,
    edit_state: EditStateMode,
}

impl<'a> VoxjFileBuilder<'a> {
    /// Starts a builder encoding `state` into a Voxel Json document.
    pub fn new(state: &'a VoxjVoxMain) -> Self {
        Self {
            state,
            position_encoding: None,
            sample_encoding: None,
            ext: true,
            edit_state: EditStateMode::Auto,
        }
    }

    /// Sets the position-block encoding, or `None` to search for the smallest
    /// paired with the sample encoding.
    pub fn position_encoding(mut self, position_encoding: Option<PositionEncoding>) -> Self {
        self.position_encoding = position_encoding;
        self
    }

    /// Sets the sample-block encoding, or `None` to search for the smallest
    /// paired with the position encoding.
    pub fn sample_encoding(mut self, sample_encoding: Option<SampleEncoding>) -> Self {
        self.sample_encoding = sample_encoding;
        self
    }

    /// Keeps (the default) or drops the carried `ext` extension block.
    pub fn ext(mut self, ext: bool) -> Self {
        self.ext = ext;
        self
    }

    /// Sets when each object's editor build volume is recorded in the
    /// document's edit state.
    pub fn edit_state(mut self, edit_state: EditStateMode) -> Self {
        self.edit_state = edit_state;
        self
    }

    /// Builds the [`VoxjFile`].
    pub fn build(self) -> Result<VoxjFile> {
        write_voxj(
            self.state,
            self.position_encoding,
            self.sample_encoding,
            self.ext,
            self.edit_state,
        )
    }
}
