use crate::{
    PalettePlan, Result, VMaxColorFormat, color_material_map, material_name, material_settings,
};
use std::collections::BTreeMap;
use vmax::{VMaxPalettePngFile, VMaxPaletteSettingsVmaxpsbFile};

/// Writes each colored plan's color image and material sidecar.
pub(crate) fn write_palette_files(
    plans: &[PalettePlan],
    palette_settings_files: &mut BTreeMap<String, VMaxPaletteSettingsVmaxpsbFile>,
    palette_png_files: &mut BTreeMap<String, VMaxPalettePngFile>,
    vmax_color_format: VMaxColorFormat,
) -> Result<()> {
    for plan in plans {
        let Some(colors) = &plan.color_table else {
            continue;
        };
        let stem = plan
            .pal
            .strip_suffix(".png")
            .expect("a colored plan names a png");
        if matches!(
            vmax_color_format,
            VMaxColorFormat::Png | VMaxColorFormat::All
        ) {
            // 256 entries: the 255 colors 0-based then a transparent terminator.
            let mut cells = colors.clone();
            cells.push([0, 0, 0, 0]);
            palette_png_files.insert(plan.pal.clone(), VMaxPalettePngFile(cells));
        }
        // The settings sidecar carries the materials, and the colors when no
        // image does. Plist mode writes no image, so even a color-only object
        // writes its colors here rather than dropping them.
        let write_sidecar =
            !plan.materials.is_empty() || matches!(vmax_color_format, VMaxColorFormat::Plist);
        if write_sidecar {
            let sidecar = format!("{stem}.settings.vmaxpsb");
            // The plist `colors` table is the 255 colors with no terminator.
            let sidecar_colors = match vmax_color_format {
                VMaxColorFormat::Png => Vec::new(),
                VMaxColorFormat::Plist | VMaxColorFormat::All => colors.clone(),
            };
            // The per-color material map Voxel Max renders from: each used
            // color cell carries a bit for the material it draws.
            let (lc, indices, current) = color_material_map(plan);
            palette_settings_files.insert(
                sidecar,
                material_settings(
                    material_name(&plan.name),
                    plan.materials.clone(),
                    sidecar_colors,
                    lc,
                    indices,
                    current,
                ),
            );
        }
    }
    Ok(())
}
