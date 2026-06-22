use crate::{Voxel, decode_morton_3d};
use std::collections::BTreeMap;
use vmax::VMaxSnapshot;

/// Voxel pitch of a chunk along each axis; chunks tile an 8×8×8 grid into a
/// 256³ model.
const CHUNK_PITCH: i32 = 32;

/// Decodes voxel `snapshots` into model space — the inverse of
/// [`encode_snapshots`](crate::encode_snapshots). Replays snapshots so the last
/// snapshot of each chunk wins, and returns voxels sorted by `(x, y, z)`.
pub fn decode_snapshots(snapshots: &[VMaxSnapshot]) -> Vec<Voxel> {
    // Keep the last snapshot per chunk (later entries overwrite earlier).
    let mut latest: BTreeMap<u32, &VMaxSnapshot> = BTreeMap::new();
    for snapshot in snapshots {
        latest.insert(snapshot.s.id.c, snapshot);
    }

    let mut voxels: BTreeMap<(i32, i32, i32), (u8, u8)> = BTreeMap::new();
    for (chunk_id, snapshot) in latest {
        let storage = &snapshot.s;
        let grid = decode_morton_3d(chunk_id);
        let base = [
            grid[0] as i32 * CHUNK_PITCH,
            grid[1] as i32 * CHUNK_PITCH,
            grid[2] as i32 * CHUNK_PITCH,
        ];
        let morton_offset = storage.st.min.get(3).copied().unwrap_or(0).max(0) as u32;

        for (slot, pair) in storage.ds.chunks_exact(2).enumerate() {
            let (material, color) = (pair[0], pair[1]);
            if color == 0 {
                continue;
            }
            let local = decode_morton_3d(morton_offset + slot as u32);
            let position = (
                base[0] + local[0] as i32,
                base[1] + local[1] as i32,
                base[2] + local[2] as i32,
            );
            voxels.insert(position, (material, color));
        }
    }

    voxels
        .into_iter()
        .map(|((x, y, z), (material_idx, color_idx))| Voxel {
            position: [x, y, z],
            material_idx,
            color_idx,
        })
        .collect()
}
