/// Raw bytes of a `*.selection.vmaxb` file: the saved voxel selection for a
/// `contents*.vmaxb` object, sharing its `contents{n}` stem (e.g.
/// `contents1.selection.vmaxb`). Payload is opaque to this crate; held verbatim
/// and round-trips byte for byte, not through serde.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxSelectionVmaxbFile(pub Vec<u8>);
