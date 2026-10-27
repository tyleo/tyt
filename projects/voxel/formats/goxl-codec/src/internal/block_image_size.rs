/// The edge length, in pixels, of a `BL16` block's `64 x 64` PNG. A block holds
/// [`GoxlBlock::SIZE`](goxl::GoxlBlock::SIZE)`^3 == 4096` voxels, one per pixel of
/// this `64 x 64` image.
pub const BLOCK_IMAGE_SIZE: u32 = 64;
