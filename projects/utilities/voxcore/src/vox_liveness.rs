use crate::BVoxVoxel;
use branded_id::U32Id;

/// A packed occupancy bitmap for a [`VoxObject`](crate::VoxObject)'s dense
/// voxel grid: one bit per grid cell, keyed by the cell's voxel id (its raster
/// index `x * Y * Z + y * Z + z`). A set bit marks a live (filled) voxel; a
/// clear bit marks empty space.
///
/// The grid allocates a voxel id for every cell, so the sample columns always
/// have a value to return. This map is what distinguishes a filled cell from
/// empty space, because a sample is always a valid cell reference and cannot
/// by itself mark a cell empty.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VoxLiveness {
    /// Bit `i % 64` of word `i / 64`, counted from the least significant bit,
    /// is the liveness of voxel id `i`. Bits at indices `>= len` are kept clear
    /// so two maps with the same length and live set compare equal.
    words: Vec<u64>,

    /// The number of grid cells the map covers (`X * Y * Z`).
    len: usize,
}

impl VoxLiveness {
    /// Creates a map covering `len` cells, all empty.
    pub fn new(len: usize) -> Self {
        Self {
            words: vec![0; len.div_ceil(64)],
            len,
        }
    }

    /// The number of live cells.
    pub fn count_live(&self) -> usize {
        self.words
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum()
    }

    /// Whether the map covers no cells.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether the cell at `id` is live (filled).
    ///
    /// # Panics
    /// Panics if `id` is outside the covered range (`>= len`).
    pub fn is_live(&self, id: U32Id<BVoxVoxel>) -> bool {
        let i = id.to_u32() as usize;
        assert!(
            i < self.len,
            "voxel id {i} is outside the {}-cell grid",
            self.len
        );
        (self.words[i / 64] >> (i % 64)) & 1 == 1
    }

    /// Iterates the ids of the live cells in ascending raster order.
    pub fn iter_live(&self) -> impl Iterator<Item = U32Id<BVoxVoxel>> + '_ {
        self.words.iter().enumerate().flat_map(|(word, &bits)| {
            BitIndices { bits }.map(move |bit| U32Id::from_u32((word * 64 + bit) as u32))
        })
    }

    /// The number of grid cells the map covers.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Sets whether the cell at `id` is live.
    ///
    /// # Panics
    /// Panics if `id` is outside the covered range (`>= len`). Enforcing this
    /// keeps every bit at an index `>= len` clear, which the derived equality,
    /// [`count_live`](Self::count_live), and [`iter_live`](Self::iter_live)
    /// rely on.
    pub fn set_live(&mut self, id: U32Id<BVoxVoxel>, live: bool) {
        let i = id.to_u32() as usize;
        assert!(
            i < self.len,
            "voxel id {i} is outside the {}-cell grid",
            self.len
        );
        let mask = 1u64 << (i % 64);
        if live {
            self.words[i / 64] |= mask;
        } else {
            self.words[i / 64] &= !mask;
        }
    }
}

/// Iterates the set-bit positions of a `u64`, least significant first.
struct BitIndices {
    bits: u64,
}

impl Iterator for BitIndices {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.bits == 0 {
            None
        } else {
            let bit = self.bits.trailing_zeros() as usize;
            self.bits &= self.bits - 1;
            Some(bit)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BVoxVoxel, VoxLiveness};
    use branded_id::U32Id;

    fn voxel_id(index: u32) -> U32Id<BVoxVoxel> {
        U32Id::from_u32(index)
    }

    #[test]
    fn sets_and_reads_bits_across_word_boundaries() {
        let mut liveness = VoxLiveness::new(130);
        assert_eq!(liveness.len(), 130);
        assert!(!liveness.is_empty());

        for index in [0, 1, 63, 64, 65, 129] {
            assert!(!liveness.is_live(voxel_id(index)));
            liveness.set_live(voxel_id(index), true);
            assert!(liveness.is_live(voxel_id(index)));
        }

        liveness.set_live(voxel_id(64), false);
        assert!(!liveness.is_live(voxel_id(64)));
        assert!(liveness.is_live(voxel_id(65)));
    }

    #[test]
    fn iter_live_yields_ascending_ids_matching_count() {
        let mut liveness = VoxLiveness::new(200);
        let live = [3, 64, 65, 130, 199];
        for index in live {
            liveness.set_live(voxel_id(index), true);
        }

        assert_eq!(liveness.count_live(), live.len());
        let got: Vec<u32> = liveness
            .iter_live()
            .map(|voxel_id| voxel_id.to_u32())
            .collect();
        assert_eq!(got, live);
    }

    #[test]
    #[should_panic]
    fn set_live_panics_outside_len_not_just_word_capacity() {
        // len 130 rounds up to a 192-bit (3-word) capacity; an id in the gap
        // [130, 192) must still panic rather than silently set a trailing bit,
        // which would corrupt count_live / iter_live / equality.
        let mut liveness = VoxLiveness::new(130);
        liveness.set_live(voxel_id(150), true);
    }

    #[test]
    fn empty_map_has_no_live_cells() {
        let liveness = VoxLiveness::new(0);
        assert!(liveness.is_empty());
        assert_eq!(liveness.count_live(), 0);
        assert_eq!(liveness.iter_live().count(), 0);
    }
}
