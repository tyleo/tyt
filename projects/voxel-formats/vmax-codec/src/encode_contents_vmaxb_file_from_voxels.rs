use crate::{VMaxVoxel, encode_vmax_snapshots};
use vmax::VMaxContentsVmaxbFile;

/// Encodes voxels into a minimal `VMaxContentsVmaxbFile` payload. Used as
/// the fallback when no preserved object state is available (e.g. a voxel
/// document authored outside Voxel Max).
pub fn encode_contents_vmaxb_file_from_voxels(
    voxels: &[VMaxVoxel],
    uuid: &str,
) -> VMaxContentsVmaxbFile {
    VMaxContentsVmaxbFile {
        snapshots: encode_vmax_snapshots(voxels),
        uuid: uuid.to_owned(),
        v: 4,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::{VMaxVoxel, decode_vmax_snapshots, encode_contents_vmaxb_file_from_voxels};

    fn voxel(x: i32, y: i32, z: i32, material_idx: u8, color_idx: u8) -> VMaxVoxel {
        VMaxVoxel {
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
        let decoded = decode_vmax_snapshots(
            &encode_contents_vmaxb_file_from_voxels(&voxels, "uuid").snapshots,
        )
        .unwrap();
        voxels.sort_by_key(|v| (v.position[0], v.position[1], v.position[2]));
        assert_eq!(decoded, voxels);
    }
}
