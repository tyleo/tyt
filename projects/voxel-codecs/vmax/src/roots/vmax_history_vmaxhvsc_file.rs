/// The raw bytes of a `*.vmaxhvsc` history voxel-snapshot sidecar: a small
/// binary plist (`bplist00`) companion to a `*.vmaxhvsb` snapshot buffer,
/// sharing its `history{n}` stem (e.g. `history1.vmaxhvsc`), that records the
/// snapshot metadata Voxel Max keeps alongside the buffer. Its contents are
/// opaque to this crate, so it is held verbatim as raw bytes and round-trips
/// byte for byte rather than through serde.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxHistoryVmaxhvscFile(pub Vec<u8>);
