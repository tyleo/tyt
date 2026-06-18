use crate::{
    VXObjectDataSerde, VXSnapshotIdSerde, VXSnapshotSerde, VXStatsSerde, VXStorageSerde,
    encode_morton_3d,
};
use std::collections::BTreeMap;
use vmax::VMaxVoxel;

/// Voxel pitch of a chunk along each axis; chunks tile an 8×8×8 grid into a
/// 256³ model.
const CHUNK_PITCH: i32 = 32;

/// Encodes voxels into a `VXObjectData` payload — the inverse of
/// [`VXObjectDataSerde::voxels`]. Groups voxels into the 32-pitch 8×8×8 chunk
/// grid, lays each chunk's voxels out by in-chunk Morton code into a dense
/// 2-bytes-per-slot `(material, color)` stream (gaps left as `color == 0`
/// empty), and emits one snapshot per occupied chunk with `st.min[3]` set to the
/// first slot's Morton code.
pub fn encode_object_data(voxels: &[VMaxVoxel]) -> VXObjectDataSerde {
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

    let snapshots = chunks
        .into_iter()
        .map(|(chunk_id, slots)| {
            let min_morton = *slots.keys().next().expect("non-empty chunk");
            let max_morton = *slots.keys().next_back().expect("non-empty chunk");
            let mut ds = vec![0u8; 2 * (max_morton - min_morton + 1) as usize];
            for (morton, (material, color)) in slots {
                let slot = (morton - min_morton) as usize;
                ds[2 * slot] = material;
                ds[2 * slot + 1] = color;
            }
            VXSnapshotSerde {
                s: VXStorageSerde {
                    id: VXSnapshotIdSerde { c: chunk_id },
                    ds,
                    st: VXStatsSerde {
                        min: vec![0, 0, 0, min_morton as i64],
                    },
                },
            }
        })
        .collect();

    VXObjectDataSerde { snapshots }
}

#[cfg(test)]
mod tests {
    use crate::encode_object_data;
    use vmax::VMaxVoxel;

    fn voxel(x: i32, y: i32, z: i32, material: u8, color: u8) -> VMaxVoxel {
        VMaxVoxel {
            x,
            y,
            z,
            material,
            color,
        }
    }

    #[test]
    fn round_trips_through_voxels() {
        // Voxels spanning several 32-pitch chunks, unsorted, distinct colors.
        let mut voxels = vec![
            voxel(70, 0, 0, 1, 5),
            voxel(0, 0, 0, 0, 7),
            voxel(35, 40, 3, 2, 200),
            voxel(5, 5, 5, 7, 1),
            voxel(255, 255, 255, 3, 9),
        ];
        let decoded = encode_object_data(&voxels).voxels();
        voxels.sort_by_key(|v| (v.x, v.y, v.z));
        assert_eq!(decoded, voxels);
    }
}
