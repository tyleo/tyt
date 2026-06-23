/// The raw bytes of a `QuickLook/*.png` thumbnail: a macOS QuickLook preview
/// Voxel Max renders for the package — the whole-scene `QuickLook/Thumbnail.png`,
/// the per-object `QuickLook/contents{n}.vmaxb.png`, and the per-group
/// `QuickLook/{group-id}.png`. Held verbatim (never re-encoded) so the
/// `vmax-codec` crate round-trips each preview byte for byte.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxQuickLookPngFile(pub Vec<u8>);
