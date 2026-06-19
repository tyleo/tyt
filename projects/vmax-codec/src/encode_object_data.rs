use crate::{VXObjectDataSerde, encode_snapshots};
use vmax::VMaxVoxel;

/// Encodes voxels into a minimal `VXObjectData` payload — voxel `snapshots` (via
/// [`encode_snapshots`]) plus the content `uuid` and current version, with no
/// editor state. Used as the fallback when no preserved
/// [`VXObjectStateSerde`](crate::VXObjectStateSerde) is available (e.g. a voxj
/// document authored outside Voxel Max); a round-tripped document instead rebuilds
/// the payload from its preserved state.
pub fn encode_object_data(voxels: &[VMaxVoxel], uuid: &str) -> VXObjectDataSerde {
    VXObjectDataSerde {
        snapshots: encode_snapshots(voxels),
        uuid: uuid.to_owned(),
        v: 4,
        ..Default::default()
    }
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
        let decoded = encode_object_data(&voxels, "uuid").voxels();
        voxels.sort_by_key(|v| (v.x, v.y, v.z));
        assert_eq!(decoded, voxels);
    }
}
