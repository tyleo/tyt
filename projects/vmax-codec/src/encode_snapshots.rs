use crate::{VMaxVoxel, decode_morton_3d, encode_morton_3d};
use std::collections::BTreeMap;
use vmax::{VXExtent, VXSnapshot, VXSnapshotId, VXStats, VXStorage};

/// Voxel pitch of a chunk along each axis; chunks tile an 8×8×8 grid into a
/// 256³ model.
const CHUNK_PITCH: i32 = 32;

/// Highest local coordinate within a chunk (32 voxels per axis).
const CHUNK_MAX: u32 = 31;

/// `st.extent.o` order tag Voxel Max writes for the 32³ chunk grid (2⁵ = 32).
const CHUNK_ORDER: i64 = 5;

/// Snapshot type written for every rebuilt chunk (4 = checkpoint).
const CHECKPOINT: i64 = 4;

/// The 4-component Morton stat Voxel Max stores for a corner in `st`: the per-axis
/// spread of the corner's local coords plus their sum (the Morton code itself, so
/// `stat[3] == stat[0] + stat[1] + stat[2]`).
fn morton_stat(morton: u32) -> Vec<i64> {
    let c = decode_morton_3d(morton);
    let x = encode_morton_3d([c[0], 0, 0]) as i64;
    let y = encode_morton_3d([0, c[1], 0]) as i64;
    let z = encode_morton_3d([0, 0, c[2]]) as i64;
    vec![x, y, z, x + y + z]
}

/// Encodes voxels into a `VXObjectData`'s `snapshots` array — the inverse of
/// [`decode_snapshots`](crate::decode_snapshots). Groups voxels
/// into the 32-pitch 8×8×8 chunk grid, lays each chunk's voxels out by in-chunk
/// Morton code into a dense 2-bytes-per-slot `(material, color)` stream (gaps left
/// as `color == 0` empty), and emits one checkpoint snapshot per occupied chunk
/// with the per-chunk statistics Voxel Max expects.
pub fn encode_snapshots(voxels: &[VMaxVoxel]) -> Vec<VXSnapshot> {
    let mut chunks: BTreeMap<u32, BTreeMap<u32, (u8, u8)>> = BTreeMap::new();
    for voxel in voxels {
        let grid = [
            (voxel.x / CHUNK_PITCH) as u32,
            (voxel.y / CHUNK_PITCH) as u32,
            (voxel.z / CHUNK_PITCH) as u32,
        ];
        let local = [
            (voxel.x % CHUNK_PITCH) as u32,
            (voxel.y % CHUNK_PITCH) as u32,
            (voxel.z % CHUNK_PITCH) as u32,
        ];
        chunks
            .entry(encode_morton_3d(grid))
            .or_default()
            .insert(encode_morton_3d(local), (voxel.material, voxel.color));
    }

    chunks
        .into_iter()
        .map(|(chunk_id, slots)| {
            let min_morton = *slots.keys().next().expect("non-empty chunk");
            let max_morton = *slots.keys().next_back().expect("non-empty chunk");
            let count = slots.len() as i64;

            let mut ds = vec![0u8; 2 * (max_morton - min_morton + 1) as usize];
            for (morton, (material, color)) in slots {
                let slot = (morton - min_morton) as usize;
                ds[2 * slot] = material;
                ds[2 * slot + 1] = color;
            }

            VXSnapshot {
                s: VXStorage {
                    id: VXSnapshotId {
                        c: chunk_id,
                        s: 0,
                        t: CHECKPOINT,
                    },
                    ds,
                    st: VXStats {
                        // Occupied Morton range; min[3]/max[3] are the `ds` bounds.
                        min: morton_stat(min_morton),
                        max: morton_stat(max_morton),
                        extent: VXExtent { o: CHUNK_ORDER },
                        count,
                        // Selection bounds span the whole 32³ chunk.
                        smin: vec![0; 4],
                        smax: morton_stat(encode_morton_3d([CHUNK_MAX; 3])),
                        scount: 0,
                    },
                    lc: vec![0u8; 256],
                    dlc: vec![0u8; 256],
                },
            }
        })
        .collect()
}
