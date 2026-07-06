use crate::{
    ColorSpace, Dither, FillMode, GridResolution, MaterialMode, MeshFormat, PaletteReduction,
    QuantizeMethod, Result, VoxjColorFormat, VoxjEncoding, VoxjFormat, implementation,
};
use std::{fs, path::Path};
use ty_math::{TyVector3, TyVector3U32};
use voxcore::VoxMain;
use voxsmith::{
    ColorSpace as VoxsmithColorSpace, Dither as VoxsmithDither, EditStateMode,
    FillMode as VoxsmithFillMode, MaterialMode as VoxsmithMaterialMode,
    ReductionMethod as VoxsmithReductionMethod, from_gltf_bytes, order_palette_colors,
    reduce_palette, voxelize_mesh,
};

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
    resolution: GridResolution,
    fill_mode: FillMode,
    material_mode: MaterialMode,
    fill_color: Option<[u8; 4]>,
    name: Option<&str>,
    reduction: PaletteReduction,
    encoding: VoxjEncoding,
    format: VoxjFormat,
    color_format: VoxjColorFormat,
) -> Result<()> {
    let bytes = fs::read(input)?;

    // Parse the mesh once, then read its extent to size the grid and rasterize
    // it, rather than parsing the glTF twice.
    let mesh = from_gltf_bytes(&bytes)?;

    let (counts, node_scale) = resolve_grid(mesh.extent(), resolution);

    // The final fallback when neither `--name` nor the glTF names the object.
    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("voxelized");

    let mut state = voxelize_mesh(
        &mesh,
        counts,
        fill_mode_mode(fill_mode),
        material_mode_mode(material_mode),
        fill_color,
        node_scale,
        name,
        stem,
    )?;

    reduce_generated_palette(&mut state, reduction)?;

    // Canonicalize the generated palette so its materials reference colors in id
    // order, `0, 1, 2, ...`, rather than the order voxelize and reduction left.
    let palette = state.iter_palettes().next().map(|(palette, _)| palette);
    if let Some(palette) = palette {
        order_palette_colors(&mut state, palette);
    }

    implementation::write_voxj_document(
        &state,
        output,
        encoding,
        format,
        color_format,
        false,
        EditStateMode::Never,
    )
}

/// Resolves the grid counts and the placing node's scale from the mesh `extent`
/// and the chosen `resolution`. `MetersPerVoxel` sizes each axis to a fixed
/// real-world voxel size and records that size as the node scale;
/// `VoxelGridLength` caps the longest axis at the given count, preserving aspect,
/// and leaves the node scale at `1`. Every axis is at least one voxel.
fn resolve_grid(extent: TyVector3<f64>, resolution: GridResolution) -> (TyVector3U32, f64) {
    match resolution {
        GridResolution::MetersPerVoxel(meters) => {
            let count = |edge: f64| (edge / meters).ceil().max(1.0) as u32;

            let counts = TyVector3U32::new(count(extent.x), count(extent.y), count(extent.z));

            (counts, meters)
        }

        GridResolution::VoxelGridLength(length) => {
            let n = length.max(1) as f64;

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
}

/// Applies the `--max-palette-materials` cap to voxelize's one generated
/// palette. Inert when the cap is `none` or the palette already fits. Reduction
/// is the designed default, so it fires on nearly every run and stays quiet.
fn reduce_generated_palette(state: &mut VoxMain, reduction: PaletteReduction) -> Result<()> {
    let Some(max_materials) = reduction.max_materials else {
        return Ok(());
    };

    let Some((palette, _)) = state.iter_palettes().next() else {
        return Ok(());
    };

    reduce_palette(
        state,
        palette,
        max_materials,
        reduction_method(reduction.method),
        color_space(reduction.space),
        dither(reduction.dither),
        reduction.keep_unused_values,
    )?;

    Ok(())
}

/// Maps a CLI fill-mode choice to the voxsmith fill mode.
fn fill_mode_mode(fill_mode: FillMode) -> VoxsmithFillMode {
    match fill_mode {
        FillMode::Solid => VoxsmithFillMode::Solid,
        FillMode::Surface => VoxsmithFillMode::Surface,
    }
}

/// Maps a CLI material-mode choice to the voxsmith material mode.
fn material_mode_mode(material_mode: MaterialMode) -> VoxsmithMaterialMode {
    match material_mode {
        MaterialMode::Auto => VoxsmithMaterialMode::Auto,
        MaterialMode::PerPrimitive => VoxsmithMaterialMode::PerPrimitive,
        MaterialMode::PerTexel => VoxsmithMaterialMode::PerTexel,
        MaterialMode::Flat => VoxsmithMaterialMode::Flat,
    }
}

/// Maps a CLI quantize method to the voxsmith reduction method.
fn reduction_method(method: QuantizeMethod) -> VoxsmithReductionMethod {
    match method {
        QuantizeMethod::MedianCut => VoxsmithReductionMethod::MedianCut,
        QuantizeMethod::Octree => VoxsmithReductionMethod::Octree,
        QuantizeMethod::Kmeans => VoxsmithReductionMethod::Kmeans,
    }
}

/// Maps a CLI color space to the voxsmith color space.
fn color_space(space: ColorSpace) -> VoxsmithColorSpace {
    match space {
        ColorSpace::Oklab => VoxsmithColorSpace::Oklab,
        ColorSpace::Lab => VoxsmithColorSpace::Lab,
        ColorSpace::Rgb => VoxsmithColorSpace::Rgb,
    }
}

/// Maps a CLI dither choice to the voxsmith dither.
fn dither(dither: Dither) -> VoxsmithDither {
    match dither {
        Dither::None => VoxsmithDither::None,
        Dither::FloydSteinberg => VoxsmithDither::FloydSteinberg,
        Dither::Ordered => VoxsmithDither::Ordered,
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_grid;
    use crate::GridResolution;
    use ty_math::{TyVector3, TyVector3U32};

    #[test]
    fn voxel_grid_length_caps_the_longest_axis_and_preserves_aspect() {
        let (counts, node_scale) = resolve_grid(
            TyVector3::new(4.0, 2.0, 1.0),
            GridResolution::VoxelGridLength(4),
        );
        assert_eq!(counts, TyVector3U32::new(4, 2, 1));
        assert_eq!(node_scale, 1.0);
    }

    #[test]
    fn voxel_grid_length_keeps_a_zero_axis_at_one_voxel() {
        let (counts, _) = resolve_grid(
            TyVector3::new(4.0, 0.0, 2.0),
            GridResolution::VoxelGridLength(4),
        );
        assert_eq!(counts, TyVector3U32::new(4, 1, 2));
    }

    #[test]
    fn meters_per_voxel_rounds_each_axis_up_and_records_the_size() {
        // 3 / 2 = 1.5 -> 2 voxels per axis.
        let (counts, node_scale) = resolve_grid(
            TyVector3::new(3.0, 4.0, 3.0),
            GridResolution::MetersPerVoxel(2.0),
        );
        assert_eq!(counts, TyVector3U32::new(2, 2, 2));
        assert_eq!(node_scale, 2.0);
    }
}
