use crate::{Error, Result};
use voxrender::RenderScene;
use voxrender_wgpu::PixelHit;

/// How a raster packs a hit into a texel's 32 bits: the voxel id in the low
/// bits, then the face, then the placement's index.
#[derive(Clone, Copy, Debug)]
pub struct HitPacking {
    /// The low bits of a packed hit that hold its voxel id.
    pub voxel_bits: u32,
}

impl HitPacking {
    /// The bits of a packed hit that hold its face.
    pub const FACE_BITS: u32 = 3;

    /// The clear's texel, which a pixel without a hit keeps.
    const NO_HIT: u32 = u32::MAX;

    /// The packing that fits every hit of `scene`. Errors unless each hit
    /// packs into 32 bits.
    pub fn new(scene: &RenderScene) -> Result<Self> {
        let placements = scene.iter_placements().count();

        let largest = scene
            .iter_placements()
            .map(|(_, placement)| {
                let bounds = scene
                    .object(placement.object_id)
                    .expect("a placement's object is one of the scene's")
                    .bounds();

                u64::from(bounds.x) * u64::from(bounds.y) * u64::from(bounds.z)
            })
            .max()
            .unwrap_or(0);

        let voxel_bits = u64::BITS - largest.saturating_sub(1).leading_zeros();
        let placement_bits = usize::BITS - placements.saturating_sub(1).leading_zeros();

        if voxel_bits + Self::FACE_BITS + placement_bits > u32::BITS {
            return Err(Error::RasterHit {
                placements,
                voxel_bits,
            });
        }

        Ok(HitPacking { voxel_bits })
    }

    /// The hit a texel's `word` holds, with no shadow bits.
    pub fn unpack(self, word: u32) -> Option<PixelHit> {
        (word != Self::NO_HIT).then(|| {
            let key = word >> self.voxel_bits;

            PixelHit {
                placement: key >> Self::FACE_BITS,
                voxel: word & ((1 << self.voxel_bits) - 1),
                face: key & ((1 << Self::FACE_BITS) - 1),
                lit: Vec::new(),
            }
        })
    }
}
