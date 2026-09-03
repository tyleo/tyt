use crate::{
    VMaxExtent, VMaxSnapshot, VMaxSnapshotId, VMaxStats, VMaxStorage,
    snapshots::{VMaxVoxel, decode_morton_3d, encode_morton_3d},
};
use std::collections::BTreeMap;

/// Chunk voxel pitch per axis; an 8x8x8 grid of chunks tiles a 256^3 model.
const CHUNK_PITCH: i32 = 32;

/// Highest local coordinate within a chunk (32 voxels per axis).
const CHUNK_MAX: u32 = 31;

/// `st.extent.o` order tag Voxel Max writes for the 32^3 chunk grid (2^5 = 32).
const CHUNK_ORDER: i64 = 5;

/// Snapshot type written for every rebuilt chunk (4 = checkpoint).
const CHECKPOINT: i64 = 4;

/// Encodes voxels into a `VMaxContentsVmaxbFile`'s `snapshots` array, the
/// inverse of
/// [`decode_vmax_snapshots`](crate::snapshots::decode_vmax_snapshots). Emits
/// one checkpoint snapshot per occupied chunk.
pub fn encode_vmax_snapshots(voxels: &[VMaxVoxel]) -> Vec<VMaxSnapshot> {
    let mut chunks: BTreeMap<u32, BTreeMap<u32, (u8, u8)>> = BTreeMap::new();
    for voxel in voxels {
        let grid = [
            (voxel.position[0] / CHUNK_PITCH) as u32,
            (voxel.position[1] / CHUNK_PITCH) as u32,
            (voxel.position[2] / CHUNK_PITCH) as u32,
        ];
        let local = [
            (voxel.position[0] % CHUNK_PITCH) as u32,
            (voxel.position[1] % CHUNK_PITCH) as u32,
            (voxel.position[2] % CHUNK_PITCH) as u32,
        ];
        chunks.entry(encode_morton_3d(grid)).or_default().insert(
            encode_morton_3d(local),
            (voxel.material_idx, voxel.color_idx),
        );
    }

    chunks
        .into_iter()
        .map(|(chunk_id, slots)| {
            // Anchor `ds` and the `st` bounds to the Morton codes of the
            // occupied bounding-box corners, not the smallest/largest occupied
            // code. Voxel Max reads these corners to place the editor grid;
            // tight occupied-code bounds would mislocate it for sparse chunks.
            let mut lo = [CHUNK_MAX; 3];
            let mut hi = [0u32; 3];
            for &morton in slots.keys() {
                let coords = decode_morton_3d(morton);
                for axis in 0..3 {
                    lo[axis] = lo[axis].min(coords[axis]);
                    hi[axis] = hi[axis].max(coords[axis]);
                }
            }
            let min_morton = encode_morton_3d(lo);
            let max_morton = encode_morton_3d(hi);
            let count = slots.len() as i64;

            let mut ds = vec![0u8; 2 * (max_morton - min_morton + 1) as usize];
            // Layer-color usage mask: one byte per palette index, set for each
            // color used in this chunk. Voxel Max indexes it at `color - 1` to
            // flag in-use cells in the color picker.
            let mut lc = vec![0u8; 256];
            for (morton, (material, color)) in slots {
                let slot = (morton - min_morton) as usize;
                ds[2 * slot] = material;
                ds[2 * slot + 1] = color;
                if color != 0 {
                    lc[(color - 1) as usize] = 1;
                }
            }

            VMaxSnapshot {
                s: VMaxStorage {
                    id: VMaxSnapshotId {
                        c: chunk_id,
                        s: 0,
                        t: CHECKPOINT,
                    },
                    ds,
                    st: VMaxStats {
                        // Occupied bounding-box corners; min[3]/max[3] are the
                        // first/last `ds` slot's Morton code.
                        min: morton_stat(min_morton),
                        max: morton_stat(max_morton),
                        extent: VMaxExtent {
                            o: CHUNK_ORDER,
                            r: None,
                        },
                        count,
                        // Selection bounds span the whole 32^3 chunk.
                        smin: vec![0; 4],
                        smax: morton_stat(encode_morton_3d([CHUNK_MAX; 3])),
                        scount: 0,
                    },
                    lc,
                    dlc: vec![0u8; 256],
                    // `is` is a legacy alternative to `lc`/`dlc`; rebuilt
                    // snapshots use the current masks, so it stays empty.
                    is: Vec::new(),
                },
            }
        })
        .collect()
}

/// The 4-component Morton stat for a corner in `st`: per-axis decomposition of
/// the corner's local coords plus their sum
/// (`stat[3] == stat[0]+stat[1]+stat[2]`).
fn morton_stat(morton: u32) -> Vec<i64> {
    let c = decode_morton_3d(morton);
    let x = encode_morton_3d([c[0], 0, 0]) as i64;
    let y = encode_morton_3d([0, c[1], 0]) as i64;
    let z = encode_morton_3d([0, 0, c[2]]) as i64;
    vec![x, y, z, x + y + z]
}

#[cfg(test)]
mod tests {
    use crate::snapshots::{VMaxVoxel, encode_vmax_snapshots};

    fn voxel(x: i32, y: i32, z: i32, color_idx: u8) -> VMaxVoxel {
        VMaxVoxel {
            position: [x, y, z],
            material_idx: 0,
            color_idx,
        }
    }

    /// A sparse chunk whose bounding-box corners are empty must still anchor
    /// `st`/`ds` to those corner Morton codes, not the occupied codes.
    #[test]
    fn anchors_stats_to_bounding_box_corners() {
        // Voxels at (1, 0, 0) and (0, 1, 0) in chunk 0: the bounding box spans
        // the (0, 0, 0)..(1, 1, 0) corners (codes 0 and 3), neither occupied;
        // the occupied codes are 1 and 2.
        let snapshots = encode_vmax_snapshots(&[voxel(1, 0, 0, 1), voxel(0, 1, 0, 2)]);
        assert_eq!(snapshots.len(), 1);
        let storage = &snapshots[0].s;

        // st.min/st.max are the corner codes (0 and 3), per-axis-decomposed.
        assert_eq!(storage.st.min, vec![0, 0, 0, 0]);
        assert_eq!(storage.st.max, vec![1, 2, 0, 3]);
        // count is still the occupied-voxel count, not the slot span.
        assert_eq!(storage.st.count, 2);

        // ds spans all four corner-to-corner slots (2 bytes each); the two
        // corner slots (0 and 3) stay empty, the occupied slots (1 and 2) carry
        // color.
        assert_eq!(storage.ds, vec![0, 0, 0, 1, 0, 2, 0, 0]);
    }
}
