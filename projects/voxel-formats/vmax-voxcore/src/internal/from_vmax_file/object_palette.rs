use crate::{
    ABSORPTION, Result, SHADOWS, VMaxExtPalette, color_cells, combo_key, dispersion,
    float_value_pool, material_list, vm_coefficient_to_pbr_factor, vmax_ext_material,
};
use branded_id::U32Id;
use std::collections::HashMap;
use ty_math::TySrgbaU8;
use vmax::{VMaxFile, VMaxObject, snapshots::VMaxVoxel};
use voxcore::{
    BVoxMaterial, BVoxPalette, VoxMain, VoxPalette, VoxValuePool,
    color::lin_srgba_f64_from_srgba_u8,
    material::{
        BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC, ROUGHNESS, TRANSMISSION,
        default_scalar,
    },
};

/// The palette built for one object and added to a [`VoxMain`], with the data
/// `build_object` needs to sample its voxels and record its ext provenance.
pub(crate) struct ObjectPalette {
    /// The palette id in the state.
    pub(crate) palette_id: U32Id<BVoxPalette>,

    /// The ext provenance carrying the name and exact material list.
    pub(crate) provenance: VMaxExtPalette,

    /// Each used color-and-material combination's material id.
    pub(crate) combo_material_ids: HashMap<(u8, u8), U32Id<BVoxMaterial>>,

    /// Whether the object carries materials.
    pub(crate) has_materials: bool,
}

/// Builds one palette for an object's live voxels, adding it and its value
/// pools to `main`. Voxel Max's color table and material list become one
/// palette: `baseColor` and `emissiveColor` ride the color axis, one value
/// per color cell, every material scalar rides the material axis, one value
/// per material slot, and one material per color-and-material combination
/// the voxels use gathers a value on each.
///
/// The color value pool is the object's full color table in order, so a
/// material's `baseColor` value-index is `color_idx - 1`. Each material
/// scalar value pool holds one value per Voxel Max material, in order; the
/// material byte is 0-based, so a voxel's value-index is its `material_idx`.
/// The exact material list rides in the ext provenance for a byte-exact
/// write-back.
pub(crate) fn object_palette(
    serde: &VMaxFile,
    object: &VMaxObject,
    voxels: &[VMaxVoxel],
    main: &mut VoxMain<()>,
) -> Result<ObjectPalette> {
    let colors = color_cells(serde, object);
    let (name, materials) = material_list(serde, object);
    let has_materials = !materials.is_empty();

    // `color_axis` records each property's axis in property order, so a
    // material gathers one value id per property.
    let mut palette = VoxPalette::default();
    let mut color_axis: Vec<bool> = Vec::new();
    let color_value_pool_id = main.retain_value_pool(VoxValuePool::vec_4_float(
        colors
            .iter()
            .map(|color| <[f64; 4]>::from(lin_srgba_f64_from_srgba_u8(TySrgbaU8::from(*color))))
            .collect(),
    )?);
    palette
        .retain_property(
            BASE_COLOR.to_owned(),
            color_value_pool_id,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");
    color_axis.push(true);

    // Metalness and roughness convert from Voxel Max's 0.1 to 0.9 slider
    // coefficient to the 0 to 1 glTF factor the value-pool name implies; see
    // [`vm_coefficient_to_pbr_factor`]. The remaining scalars are unbounded and
    // stay raw: `sic` is an unbounded emission strength, and `shadows` and
    // `absorption` have no glTF counterpart. The exact coefficients ride in the
    // ext for a byte-exact write-back.
    if has_materials {
        let metallic_value_pool_id = float_value_pool(
            main,
            materials
                .iter()
                .map(|m| vm_coefficient_to_pbr_factor(m.mc))
                .collect(),
        )?;
        palette
            .retain_property(
                METALLIC.to_owned(),
                metallic_value_pool_id,
                U32Id::from_u32(0),
            )
            .expect("the property names are distinct");
        color_axis.push(false);
        let roughness_value_pool_id = float_value_pool(
            main,
            materials
                .iter()
                .map(|m| vm_coefficient_to_pbr_factor(m.rc))
                .collect(),
        )?;
        palette
            .retain_property(
                ROUGHNESS.to_owned(),
                roughness_value_pool_id,
                U32Id::from_u32(0),
            )
            .expect("the property names are distinct");
        color_axis.push(false);
        // Voxel Max glows in the voxel's own base color, so an emissive
        // material's color is its base color. The property appears only when
        // some material emits, and rides the color axis like `baseColor`; the
        // emissive is then `emissiveFactor` times `emissiveStrength` per glTF, so
        // the color leads the strength that scales it.
        if materials.iter().any(|m| m.sic > 0.0) {
            let emissive_color_value_pool_id = main.retain_value_pool(VoxValuePool::vec_3_float(
                colors
                    .iter()
                    .map(|color| {
                        let linear = lin_srgba_f64_from_srgba_u8(TySrgbaU8::from(*color));
                        [linear.red, linear.green, linear.blue]
                    })
                    .collect(),
            )?);
            palette
                .retain_property(
                    EMISSIVE_COLOR.to_owned(),
                    emissive_color_value_pool_id,
                    U32Id::from_u32(0),
                )
                .expect("the property names are distinct");
            color_axis.push(true);
        }
        let emissive_value_pool_id =
            float_value_pool(main, materials.iter().map(|m| m.sic).collect())?;
        palette
            .retain_property(
                EMISSIVE_STRENGTH.to_owned(),
                emissive_value_pool_id,
                U32Id::from_u32(0),
            )
            .expect("the property names are distinct");
        color_axis.push(false);

        // Dispersion properties appear only when some material carries an `md`
        // block. A material without one takes the glTF default ior and zero
        // transmission and absorption; its absence rides in the ext.
        if materials.iter().any(|m| m.md.is_some()) {
            let default_ior = default_scalar(IOR).expect("ior has a glTF default");
            let ior_value_pool_id = float_value_pool(
                main,
                materials
                    .iter()
                    .map(|m| m.md.as_ref().map_or(default_ior, |d| d.ior))
                    .collect(),
            )?;
            palette
                .retain_property(IOR.to_owned(), ior_value_pool_id, U32Id::from_u32(0))
                .expect("the property names are distinct");
            color_axis.push(false);
            let transmission_value_pool_id =
                float_value_pool(main, dispersion(&materials, |d| d.transmission))?;
            palette
                .retain_property(
                    TRANSMISSION.to_owned(),
                    transmission_value_pool_id,
                    U32Id::from_u32(0),
                )
                .expect("the property names are distinct");
            color_axis.push(false);
            let absorption_value_pool_id =
                float_value_pool(main, dispersion(&materials, |d| d.absorption))?;
            palette
                .retain_property(
                    ABSORPTION.to_owned(),
                    absorption_value_pool_id,
                    U32Id::from_u32(0),
                )
                .expect("the property names are distinct");
            color_axis.push(false);
        }

        let shadows_value_pool_id = main.retain_value_pool(VoxValuePool::boolean(
            materials.iter().map(|m| m.sh).collect(),
        ));
        palette
            .retain_property(
                SHADOWS.to_owned(),
                shadows_value_pool_id,
                U32Id::from_u32(0),
            )
            .expect("the property names are distinct");
        color_axis.push(false);
    }

    // One material per distinct combination, ordered by color cell then
    // material byte, so the material rows are canonical rather than voxel-scan
    // order and the round-trip is stable. The color index is 1-based in Voxel
    // Max, so a color-axis property takes `color_idx - 1`; the material byte
    // is 0-based, so every material-axis property takes it directly.
    let mut keys: Vec<(u8, u8)> = voxels
        .iter()
        .map(|voxel| combo_key(voxel, has_materials))
        .collect();
    keys.sort_unstable();
    keys.dedup();
    let mut combo_material_ids: HashMap<(u8, u8), U32Id<BVoxMaterial>> = HashMap::new();
    for key in keys {
        let color_index = u32::from(key.0).saturating_sub(1);
        let material_index = u32::from(key.1);
        let value_ids = color_axis
            .iter()
            .map(|&is_color| {
                U32Id::from_u32(if is_color {
                    color_index
                } else {
                    material_index
                })
            })
            .collect();
        let material_id = palette
            .retain_material(value_ids)
            .expect("one value id per property");
        combo_material_ids.insert(key, material_id);
    }

    let palette_id = main.retain_palette(palette)?;
    let provenance = VMaxExtPalette {
        name,
        materials: materials.iter().map(vmax_ext_material).collect(),
    };
    Ok(ObjectPalette {
        palette_id,
        provenance,
        combo_material_ids,
        has_materials,
    })
}
