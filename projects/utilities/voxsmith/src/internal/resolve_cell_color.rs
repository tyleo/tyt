use crate::{BASE_COLOR_FACTOR, CellColor, Error, Result, value_pool_color};
use branded_id::IdVec;
use voxcore::{VoxMain, VoxObject};

/// The [`CellColor`] read for `object`, picked from its winning
/// `baseColorFactor` supplier in the effective palette: a table of
/// per-material colors, read through the material each voxel samples in the
/// winning layer. All decoding happens here. A supplier drawing from a
/// non-color value pool errors, as does a layer referencing a palette the
/// state does not hold. `Ok(None)` when no layer supplies `baseColorFactor`.
pub fn resolve_cell_color<'a>(
    state: &VoxMain,
    object: &'a VoxObject,
) -> Result<Option<CellColor<'a>>> {
    let effective = state.effective_palette(object)?;
    let Some(property_id) = effective.property_id_by_name(BASE_COLOR_FACTOR) else {
        return Ok(None);
    };
    let property = effective
        .property(property_id)
        .expect("a resolved name identifies one of the effective palette's properties");

    let value_pool = property.value_pool();
    let mut colors = IdVec::default();
    for (material_id, value_id) in property.iter_values() {
        let color =
            value_pool_color(value_pool, value_id).ok_or_else(|| non_color_value_pool(object))?;
        let slot_id = material_id.to_usize_id();
        if slot_id.to_usize() >= colors.len() {
            colors.resize(slot_id.to_usize() + 1, [0, 0, 0, 0]);
        }

        colors[slot_id] = color;
    }

    Ok(Some(CellColor::new(object, property.layer_id(), colors)))
}

/// The error for a `baseColorFactor` drawing from a non-color value pool.
fn non_color_value_pool(object: &VoxObject) -> Error {
    Error::invalid(format!(
        "object {:?}: baseColorFactor draws from a non-color value pool",
        object.name()
    ))
}

#[cfg(test)]
mod tests {
    use crate::{BASE_COLOR_FACTOR, resolve_cell_color};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{VoxBound, VoxMain, VoxObject, VoxPalette, VoxValuePool};

    #[test]
    fn a_supplier_reads_each_voxel_through_its_material() {
        let mut state = VoxMain::default();
        let value_pool_id = state.add_value_pool(
            VoxValuePool::srgba(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]]).unwrap(),
        );

        let mut palette = VoxPalette::default();
        palette
            .add_property(
                BASE_COLOR_FACTOR.to_owned(),
                value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        let red_id = palette.add_material(vec![U32Id::from_u32(0)]).unwrap();
        let blue_id = palette.add_material(vec![U32Id::from_u32(1)]).unwrap();
        let palette_id = state.add_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.add_layer(palette_id, red_id);
        for (x, material_id) in [(0, red_id), (1, blue_id)] {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[material_id]).unwrap();
        }

        let cell_color = resolve_cell_color(&state, &object).unwrap().unwrap();
        let at = |x| {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            cell_color.color(voxel_id)
        };
        assert_eq!(at(0), [255, 0, 0, 255]);
        assert_eq!(at(1), [0, 0, 255, 255]);
    }

    #[test]
    fn none_when_no_layer_supplies_the_color() {
        let mut state = VoxMain::default();
        let mut palette = VoxPalette::default();
        palette.add_material(vec![]).unwrap();
        let palette_id = state.add_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.add_layer(palette_id, U32Id::from_u32(0));

        assert!(resolve_cell_color(&state, &object).unwrap().is_none());
    }

    #[test]
    fn a_supplier_over_a_non_color_value_pool_errors() {
        let mut state = VoxMain::default();
        let value_pool_id = state.add_value_pool(
            VoxValuePool::float(VoxBound::None, VoxBound::None, vec![1.0]).unwrap(),
        );

        let mut palette = VoxPalette::default();
        palette
            .add_property(
                BASE_COLOR_FACTOR.to_owned(),
                value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();
        let material_id = palette.add_material(vec![U32Id::from_u32(0)]).unwrap();
        let palette_id = state.add_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.add_layer(palette_id, material_id);

        assert!(resolve_cell_color(&state, &object).is_err());
    }
}
