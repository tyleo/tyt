/// The raw bytes of a `QuickLook/*.png` thumbnail: a macOS QuickLook preview
/// Voxel Max renders for the package. Held verbatim (never re-encoded) so each
/// preview round-trips byte for byte.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxQuickLookPngFile(pub Vec<u8>);
