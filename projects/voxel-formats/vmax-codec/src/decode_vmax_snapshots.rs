use crate::{Error, Result, VMaxVoxel, decode_morton_3d};
use std::collections::BTreeMap;
use vmax::VMaxSnapshot;

/// Voxel pitch of a chunk along each axis; chunks tile an 8x8x8 grid into a
/// 256^3 model.
const CHUNK_PITCH: i32 = 32;

/// Decodes voxel `snapshots` into model space, the inverse of
/// [`encode_vmax_snapshots`](crate::encode_vmax_snapshots). Replays snapshots
/// so the last snapshot of each chunk wins, and returns voxels sorted by
/// `(x, y, z)`.
pub fn decode_vmax_snapshots(snapshots: &[VMaxSnapshot]) -> Result<Vec<VMaxVoxel>> {
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
        // `st.min[3]` is the Morton code of the chunk's first `ds` slot; it
        // must be present and non-negative, or the chunk's voxels can't be
        // placed.
        let morton_offset = match storage.st.min.get(3) {
            Some(&offset) if offset >= 0 => offset as u32,
            _ => {
                return Err(Error::Invalid(format!(
                    "snapshot chunk {chunk_id} has invalid st.min {:?}; expected a 4-component Morton stat with a non-negative offset",
                    storage.st.min
                )));
            }
        };

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

    Ok(voxels
        .into_iter()
        .map(|((x, y, z), (material_idx, color_idx))| VMaxVoxel {
            position: [x, y, z],
            material_idx,
            color_idx,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::{VMaxVoxel, decode_vmax_snapshots, encode_vmax_snapshots};

    fn voxel(x: i32, y: i32, z: i32) -> VMaxVoxel {
        VMaxVoxel {
            position: [x, y, z],
            material_idx: 0,
            color_idx: 1,
        }
    }

    /// A snapshot whose `st.min` lacks its 4th (Morton-offset) component can't
    /// place the chunk's voxels, so decoding rejects it instead of assuming 0.
    #[test]
    fn rejects_missing_morton_offset() {
        let mut snapshots = encode_vmax_snapshots(&[voxel(1, 2, 3)]);
        snapshots[0].s.st.min.truncate(3);
        assert!(decode_vmax_snapshots(&snapshots).is_err());
    }

    /// A negative Morton offset is likewise rejected rather than clamped to 0.
    #[test]
    fn rejects_negative_morton_offset() {
        let mut snapshots = encode_vmax_snapshots(&[voxel(1, 2, 3)]);
        snapshots[0].s.st.min[3] = -1;
        assert!(decode_vmax_snapshots(&snapshots).is_err());
    }
}
