use crate::{Voxel, encode_snapshots};
use vmax::VMaxContentsVmaxbFile;

/// Encodes voxels into a minimal `VMaxContentsVmaxbFile` payload — voxel `snapshots`
/// (via [`encode_snapshots`]) plus the content `uuid` and current version, with
/// no editor state. Used as the fallback when no preserved object state is
/// available (e.g. a voxj document authored outside Voxel Max); a round-tripped
/// document instead rebuilds the payload from its preserved state.
pub fn encode_object_data(voxels: &[Voxel], uuid: &str) -> VMaxContentsVmaxbFile {
    VMaxContentsVmaxbFile {
        snapshots: encode_snapshots(voxels),
        uuid: uuid.to_owned(),
        v: 4,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Voxel, decode_snapshots, encode_object_data};

    fn voxel(x: i32, y: i32, z: i32, material_idx: u8, color_idx: u8) -> Voxel {
        Voxel {
            position: [x, y, z],
            material_idx,
            color_idx,
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
        let decoded = decode_snapshots(&encode_object_data(&voxels, "uuid").snapshots);
        voxels.sort_by_key(|v| (v.position[0], v.position[1], v.position[2]));
        assert_eq!(decoded, voxels);
    }
}
