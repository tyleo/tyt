/// The raw bytes of a `*.selection.vmaxb` file: the editor's saved voxel
/// selection for a `contents*.vmaxb` object, sharing its `contents{n}` stem (e.g.
/// `contents1.selection.vmaxb`). Its payload is opaque to this crate, so it is
/// held verbatim and the `vmax-codec` crate round-trips it byte for byte rather
/// than through serde.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxSelectionVmaxbFile(pub Vec<u8>);
