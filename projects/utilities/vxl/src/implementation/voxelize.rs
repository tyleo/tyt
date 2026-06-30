use crate::{FillMode, MeshFormat, Result, VoxjEncoding, VoxjFormat, implementation};
use std::{fs, path::Path};
use ty_math::{TyVector3, TyVector3U32};
use voxsmith::{EditStateMode, FillMode as VoxsmithFillMode, gltf_mesh_extent, voxelize_gltf};

/// Voxelizes the glTF or GLB mesh at `input` into a Voxel Json document at
/// `output`. The mesh extent is read once to resolve the grid counts here, then
/// voxsmith rasterizes the mesh into a [`VoxMain`](voxcore::VoxMain) that the
/// shared writer encodes. No `ext` block or edit state is recorded: a voxelized
/// mesh has neither a source `ext` to carry nor an editor build volume.
#[allow(clippy::too_many_arguments)]
pub fn voxelize(
    input: &Path,
    _from: Option<MeshFormat>,
    output: &Path,
    side_length: Option<u32>,
    scale: Option<f64>,
    fill_mode: FillMode,
    fill_color: [u8; 4],
    encoding: VoxjEncoding,
    format: VoxjFormat,
) -> Result<()> {
    let bytes = fs::read(input)?;
    let extent = gltf_mesh_extent(&bytes)?;
    let (counts, node_scale) = resolve_grid(extent, side_length, scale);
    let state = voxelize_gltf(
        &bytes,
        counts,
        fill_mode_mode(fill_mode),
        fill_color,
        node_scale,
    )?;
    implementation::write_voxj_document(
        &state,
        output,
        encoding,
        format,
        false,
        EditStateMode::Never,
    )
}

/// Resolves the grid counts and the placing node's scale from the mesh `extent`
/// and the chosen resolution. `--scale` sizes each axis to a fixed real-world
/// voxel size and records that size as the node scale; `--side-length` caps the
/// longest axis at the given count, preserving aspect, and leaves the scale at
/// `1`. Exactly one is set; `scale` wins if both somehow are. Every axis is at
/// least one voxel.
fn resolve_grid(
    extent: TyVector3<f64>,
    side_length: Option<u32>,
    scale: Option<f64>,
) -> (TyVector3U32, f64) {
    if let Some(meters) = scale {
        let count = |edge: f64| (edge / meters).ceil().max(1.0) as u32;
        let counts = TyVector3U32::new(count(extent.x), count(extent.y), count(extent.z));
        (counts, meters)
    } else {
        let n = side_length.unwrap_or(1).max(1) as f64;
        let longest = extent.x.max(extent.y).max(extent.z);
        let count = |edge: f64| {
            if longest > 0.0 {
                (edge / longest * n).round().max(1.0) as u32
            } else {
                1
            }
        };
        let counts = TyVector3U32::new(count(extent.x), count(extent.y), count(extent.z));
        (counts, 1.0)
    }
}

/// Maps a CLI fill-mode choice to the voxsmith fill mode.
fn fill_mode_mode(fill_mode: FillMode) -> VoxsmithFillMode {
    match fill_mode {
        FillMode::Solid => VoxsmithFillMode::Solid,
        FillMode::Surface => VoxsmithFillMode::Surface,
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_grid;
    use ty_math::{TyVector3, TyVector3U32};

    #[test]
    fn side_length_caps_the_longest_axis_and_preserves_aspect() {
        let (counts, scale) = resolve_grid(TyVector3::new(4.0, 2.0, 1.0), Some(4), None);
        assert_eq!(counts, TyVector3U32::new(4, 2, 1));
        assert_eq!(scale, 1.0);
    }

    #[test]
    fn side_length_keeps_a_zero_axis_at_one_voxel() {
        let (counts, _) = resolve_grid(TyVector3::new(4.0, 0.0, 2.0), Some(4), None);
        assert_eq!(counts, TyVector3U32::new(4, 1, 2));
    }

    #[test]
    fn scale_rounds_each_axis_up_and_records_the_size() {
        // 3 / 2 = 1.5 -> 2 voxels per axis.
        let (counts, scale) = resolve_grid(TyVector3::new(3.0, 4.0, 3.0), None, Some(2.0));
        assert_eq!(counts, TyVector3U32::new(2, 2, 2));
        assert_eq!(scale, 2.0);
    }
}
