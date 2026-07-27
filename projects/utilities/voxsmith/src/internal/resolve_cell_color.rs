use crate::{CellColor, Error, Result, base_color_factor_ref, pool_color};
use std::collections::HashMap;
use voxcore::{VoxMain, VoxObject};

/// The [`CellColor`] read for `object`, picked from its winning
/// `baseColorFactor` supplier: a table of per-material colors, read through
/// the material each voxel samples in the winning layer. All decoding
/// happens here; a supplier drawing from a non-color pool errors. `Ok(None)`
/// when no layer supplies `baseColorFactor`.
pub fn resolve_cell_color<'a>(
    state: &VoxMain,
    object: &'a VoxObject,
) -> Result<Option<CellColor<'a>>> {
    let Some(winner) = base_color_factor_ref(state, object) else {
        return Ok(None);
    };

    let palette_ref = state
        .palette(winner.palette)
        .expect("the winning layer references one of the state's palettes");

    let colors: HashMap<_, _> = palette_ref
        .iter_materials()
        .map(|material| {
            let (pool, value_id) = state
                .material_value(winner.palette, material, winner.property)
                .expect("a palette material carries a value for each property");
            let color = pool_color(pool, value_id).ok_or_else(|| non_color_pool(object))?;
            Ok((material, color))
        })
        .collect::<Result<_>>()?;

    Ok(Some(Box::new(move |voxel| {
        let material = object
            .voxel_material(voxel, winner.layer)
            .expect("a live voxel samples the winning layer");
        *colors
            .get(&material)
            .expect("a sampled material is one of the palette's")
    })))
}

/// The error for a `baseColorFactor` drawing from a non-color pool.
fn non_color_pool(object: &VoxObject) -> Error {
    Error::invalid(format!(
        "object {:?}: baseColorFactor draws from a non-color pool",
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
        let pool = state.add_value_pool(VoxValuePool::srgba(vec![
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
        ]));

        let mut palette = VoxPalette::default();
        palette
            .add_property(BASE_COLOR_FACTOR.to_owned(), pool)
            .unwrap();
        let red = palette.add_material(vec![U32Id::from_u32(0)]).unwrap();
        let blue = palette.add_material(vec![U32Id::from_u32(1)]).unwrap();
        let palette_id = state.add_palette(palette);

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.add_layer(palette_id, red);
        for (x, material) in [(0, red), (1, blue)] {
            let voxel = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel, &[material]).unwrap();
        }

        let cell_color = resolve_cell_color(&state, &object).unwrap().unwrap();
        let at = |x| {
            let voxel = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            cell_color(voxel)
        };
        assert_eq!(at(0), [255, 0, 0, 255]);
        assert_eq!(at(1), [0, 0, 255, 255]);
    }

    #[test]
    fn none_when_no_layer_supplies_the_color() {
        let mut state = VoxMain::default();
        let mut palette = VoxPalette::default();
        palette.add_material(vec![]).unwrap();
        let palette_id = state.add_palette(palette);

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.add_layer(palette_id, U32Id::from_u32(0));

        assert!(resolve_cell_color(&state, &object).unwrap().is_none());
    }

    #[test]
    fn a_supplier_over_a_non_color_pool_errors() {
        let mut state = VoxMain::default();
        let pool = state.add_value_pool(VoxValuePool::float(
            VoxBound::None,
            VoxBound::None,
            vec![1.0],
        ));

        let mut palette = VoxPalette::default();
        palette
            .add_property(BASE_COLOR_FACTOR.to_owned(), pool)
            .unwrap();
        let material = palette.add_material(vec![U32Id::from_u32(0)]).unwrap();
        let palette_id = state.add_palette(palette);

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.add_layer(palette_id, material);

        assert!(resolve_cell_color(&state, &object).is_err());
    }
}
