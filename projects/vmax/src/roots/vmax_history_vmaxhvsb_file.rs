/// The raw bytes of a `*.vmaxhvsb` history voxel-snapshot buffer:
/// an LZFSE-framed (`bvx2`) companion to a `*.vmaxhb` undo history, sharing its
/// `history{n}` stem (e.g. `history1.vmaxhvsb`), that holds the voxel snapshots
/// the history steps reference. Its internals are opaque to this crate, so it
/// is held verbatim as raw bytes and round-trips byte for byte rather than
/// through serde.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxHistoryVmaxhvsbFile(pub Vec<u8>);
