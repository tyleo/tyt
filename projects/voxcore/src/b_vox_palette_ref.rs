/// Brand marker for a palette reference in a [`VoxObject`](crate::VoxObject):
/// one of the object's references to a shared [`VoxPalette`](crate::VoxPalette),
/// in resolution order. Each voxel carries one sample per palette reference.
///
/// Used only as a type parameter (e.g. `IdStruct<BVoxPaletteRef>`); it is never
/// instantiated.
pub struct BVoxPaletteRef;
