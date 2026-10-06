use crate::{
    VMaxLegacyChunkVoxels,
    snapshots::{CHUNK_PITCH, Error, Result, VMaxVoxel, decode_morton_3d},
};
use std::collections::BTreeMap;

/// The bytes a legacy chunk record takes: Swift's 16-byte stride for three or
/// four 32-bit integers.
const RECORD_BYTES: usize = 16;

/// Decodes the voxels of a contents file older than version 4, as Voxel Max
/// loads one. Each 16-byte record in `chunks` holds a voxel origin inside its
/// chunk as little-endian 32-bit integers, and from version 1 a fourth integer
/// of 1 marks a chunk of two-byte voxels. Each entry of `voxels` fills its
/// chunk in Morton order: one color byte per voxel, or a material byte and a
/// color in a marked chunk. A color of 0 is empty, and a later record of a
/// chunk wins. Errors when the records and the voxel entries do not pair up.
/// Returns voxels sorted by `(x, y, z)`.
pub fn decode_vmax_legacy_chunks(
    version: i64,
    chunks: &[u8],
    voxels: &[VMaxLegacyChunkVoxels],
) -> Result<Vec<VMaxVoxel>> {
    let (records, remainder) = chunks.as_chunks::<RECORD_BYTES>();
    if !remainder.is_empty() {
        return Err(Error::Invalid(format!(
            "legacy chunk records span {} bytes, not a whole number of {RECORD_BYTES}-byte records",
            chunks.len()
        )));
    }
    if records.len() != voxels.len() {
        return Err(Error::Invalid(format!(
            "{} legacy chunk records pair with {} voxel entries",
            records.len(),
            voxels.len()
        )));
    }

    let mut decoded: BTreeMap<(i32, i32, i32), (u8, u8)> = BTreeMap::new();
    for (record, VMaxLegacyChunkVoxels(data)) in records.iter().zip(voxels) {
        let (integers, _) = record.as_chunks::<4>();
        let integer = |index: usize| i32::from_le_bytes(integers[index]);
        let base = [0, 1, 2].map(|axis| integer(axis).div_euclid(CHUNK_PITCH) * CHUNK_PITCH);
        let mut place = |offset: usize, material: u8, color: u8| {
            if color == 0 {
                return;
            }
            let local = decode_morton_3d(offset as u32);
            let position = (
                base[0] + local[0] as i32,
                base[1] + local[1] as i32,
                base[2] + local[2] as i32,
            );
            decoded.insert(position, (material, color));
        };
        if version >= 1 && integer(3) == 1 {
            for (offset, &[material, color]) in data.as_chunks::<2>().0.iter().enumerate() {
                place(offset, material, color);
            }
        } else {
            for (offset, &color) in data.iter().enumerate() {
                place(offset, 0, color);
            }
        }
    }

    Ok(decoded
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
    use crate::{
        VMaxLegacyChunkVoxels,
        snapshots::{VMaxVoxel, decode_vmax_legacy_chunks},
    };

    /// A 16-byte chunk record: three origin integers and the flag.
    fn record(origin: [i32; 3], flag: i32) -> Vec<u8> {
        origin
            .into_iter()
            .chain([flag])
            .flat_map(i32::to_le_bytes)
            .collect()
    }

    fn voxel(position: [i32; 3], material_idx: u8, color_idx: u8) -> VMaxVoxel {
        VMaxVoxel {
            position,
            material_idx,
            color_idx,
        }
    }

    /// A version 0 chunk holds one color byte per voxel in Morton order, where
    /// offset 1 steps along x and offset 4 along z, all on material slot 0.
    #[test]
    fn decodes_a_chunk_of_color_bytes() {
        let mut data = vec![0u8; 32 * 32 * 32];
        data[1] = 5;
        data[4] = 7;

        let voxels =
            decode_vmax_legacy_chunks(0, &record([0, 32, 0], 0), &[VMaxLegacyChunkVoxels(data)])
                .unwrap();

        assert_eq!(voxels, [voxel([0, 32, 1], 0, 7), voxel([1, 32, 0], 0, 5)]);
    }

    /// From version 1 a record flagged 1 holds a material byte and a color per
    /// voxel, where offset 2 steps along y.
    #[test]
    fn decodes_a_chunk_of_two_byte_voxels() {
        let mut data = vec![0u8; 2 * 32 * 32 * 32];
        data[4] = 3;
        data[5] = 9;

        let voxels =
            decode_vmax_legacy_chunks(2, &record([32, 0, 0], 1), &[VMaxLegacyChunkVoxels(data)])
                .unwrap();

        assert_eq!(voxels, [voxel([32, 1, 0], 3, 9)]);
    }

    /// Records and voxel entries pair one to one, so a count that differs
    /// errors rather than placing a chunk's voxels at another's origin.
    #[test]
    fn records_and_voxel_entries_must_pair() {
        let records = [record([0, 0, 0], 0), record([32, 0, 0], 0)].concat();

        let error = decode_vmax_legacy_chunks(1, &records, &[VMaxLegacyChunkVoxels(vec![0; 8])])
            .unwrap_err();

        assert!(error.to_string().contains("pair"), "{error}");
    }
}
