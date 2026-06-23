/// The raw bytes of a `*.vmaxhb` undo-history file: an LZFSE-framed (`bvx2`)
/// stream Voxel Max writes per object (named by the object's `hist` reference,
/// e.g. `history1.vmaxhb`) and once for the working scene (`scene.vmaxhb`). The
/// history stream's internals are opaque to this crate, so it is held verbatim
/// and the `vmax-codec` crate round-trips it byte for byte rather than through
/// serde.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxHistoryVmaxhbFile(pub Vec<u8>);
