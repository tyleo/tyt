use crate::{
    Dependencies, Format, Result, VoxDocumentFile,
    ext::{VoxconvExt, VoxconvVoxMain},
};
use voxcore::VoxMapEntry;

/// A format's typed path: the read that keeps its ext, the write that draws
/// on it, and its slot of a Voxel Json `ext` block. Each format feature adds
/// one impl. The visitors reach it through
/// [`InstalledFormat`](crate::InstalledFormat).
pub trait FormatExt: Format {
    /// Decodes a document's files into a state carrying the format's ext,
    /// boxed.
    fn read_with_ext<D: Dependencies>(
        dependencies: &D,
        files: &[VoxDocumentFile],
    ) -> Result<VoxconvVoxMain>;

    /// Encodes a state as a document's files. A box holding the format's ext
    /// writes the loaded file back exactly. Any other box encodes to its
    /// `ext` block, and the format takes its slot from that block. A block
    /// with no slot for the format writes a file synthesized from the scene.
    fn write_with_ext<D: Dependencies>(
        dependencies: &D,
        options: &Self::WriteOptions,
        state: VoxconvVoxMain,
    ) -> Result<Vec<VoxDocumentFile>>;

    /// Appends to `slots` the Voxel Json `ext` block slots `ext` encodes to
    /// when `ext` is one of this format's exts, and reports whether it was.
    /// Errors when a slot's key is already taken.
    fn encode_slots(ext: &dyn VoxconvExt, slots: &mut Vec<VoxMapEntry>) -> Result<bool>;

    /// The ext `slot` decodes to, boxed, or `None` when its key is not this
    /// format's.
    fn decode_slot(slot: &VoxMapEntry) -> Result<Option<Box<dyn VoxconvExt>>>;
}
