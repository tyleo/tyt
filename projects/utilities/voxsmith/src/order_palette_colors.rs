use crate::BASE_COLOR;
use branded_id::U32Id;
use std::collections::HashSet;
use voxcore::{BVoxPalette, BVoxValuePoolValue, VoxMain};

/// Reorders `palette_id`'s `baseColor` colors to material order: each
/// material's color in turn, then the colors no material uses. Rendering is
/// unchanged. A no-op without a `baseColor` property.
///
/// Requires a referentially valid state, which
/// [`VoxMain::validate`](voxcore::VoxMain::validate) checks.
pub fn order_palette_colors(state: &mut VoxMain, palette_id: U32Id<BVoxPalette>) {
    let Some(palette_ref) = state.palette(palette_id) else {
        return;
    };
    let Some(color_id) = palette_ref.property_id_by_name(BASE_COLOR) else {
        return;
    };
    let Some(value_pool_id) = palette_ref
        .property(color_id)
        .map(|property| property.value_pool_id)
    else {
        return;
    };
    let Some(value_pool) = state.value_pool(value_pool_id) else {
        return;
    };

    // The used color value ids in material order, then the unused ones in their
    // current order, forming a permutation of the value pool.
    let mut new_order: Vec<U32Id<BVoxValuePoolValue>> = Vec::with_capacity(value_pool.len());
    let mut seen: HashSet<U32Id<BVoxValuePoolValue>> = HashSet::with_capacity(value_pool.len());
    for material_id in palette_ref.iter_materials() {
        let Some(value_id) = palette_ref.value_id(material_id, color_id) else {
            continue;
        };
        if seen.insert(value_id) {
            new_order.push(value_id);
        }
    }
    for (value_id, _) in value_pool.iter_values() {
        if !seen.contains(&value_id) {
            new_order.push(value_id);
        }
    }

    state
        .reorder_value_pool(value_pool_id, &new_order)
        .expect("the color value pool is live and new_order is a permutation by construction");
}

#[cfg(test)]
mod tests {
    use crate::{BASE_COLOR, order_palette_colors};
    use branded_id::U32Id;
    use voxcore::{VoxMain, VoxPalette, VoxValuePool};

    #[test]
    fn orders_colors_to_material_order() {
        let mut state = VoxMain::default();
        // Three colors; materials reference them out of order: blue, red,
        // green.
        let value_pool_id = state.retain_value_pool(
            VoxValuePool::vec_4_float(vec![
                [1.0, 0.0, 0.0, 1.0], // 0 red
                [0.0, 1.0, 0.0, 1.0], // 1 green
                [0.0, 0.0, 1.0, 1.0], // 2 blue
            ])
            .unwrap(),
        );
        let mut palette = VoxPalette::default();
        let color_property_id = palette
            .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        let blue_id = palette.retain_material(vec![U32Id::from_u32(2)]).unwrap();
        let red_id = palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();
        let green_id = palette.retain_material(vec![U32Id::from_u32(1)]).unwrap();
        let palette_id = state.retain_palette(palette).unwrap();
        state.validate().unwrap();

        order_palette_colors(&mut state, palette_id);

        // The value pool lists the colors in material order. Ids and what each
        // material resolves to are unchanged.
        assert_eq!(
            state.value_pool(value_pool_id),
            Some(
                &VoxValuePool::vec_4_float(vec![
                    [0.0, 0.0, 1.0, 1.0],
                    [1.0, 0.0, 0.0, 1.0],
                    [0.0, 1.0, 0.0, 1.0],
                ])
                .unwrap()
            )
        );
        state.validate().unwrap();

        // After a gc the material color value ids follow the listing: 0, 1, 2.
        state.gc();
        let palette = state.palette(palette_id).unwrap();
        assert_eq!(
            palette.value_id(blue_id, color_property_id),
            Some(U32Id::from_u32(0))
        );
        assert_eq!(
            palette.value_id(red_id, color_property_id),
            Some(U32Id::from_u32(1))
        );
        assert_eq!(
            palette.value_id(green_id, color_property_id),
            Some(U32Id::from_u32(2))
        );
        state.validate().unwrap();
    }

    #[test]
    fn orders_colors_past_an_unused_color_and_a_hole() {
        let mut state = VoxMain::default();
        let value_pool_id = state.retain_value_pool(
            VoxValuePool::vec_4_float(vec![
                [1.0, 0.0, 0.0, 1.0], // 0 red
                [0.0, 1.0, 0.0, 1.0], // 1 green
                [0.0, 0.0, 1.0, 1.0], // 2 blue
                [1.0, 1.0, 1.0, 1.0], // 3 white, drawn by no material
                [0.0, 0.0, 0.0, 1.0], // 4 black, released below
            ])
            .unwrap(),
        );
        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![U32Id::from_u32(2)]).unwrap();
        palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();
        palette.retain_material(vec![U32Id::from_u32(1)]).unwrap();
        let palette_id = state.retain_palette(palette).unwrap();

        // Hole the value pool, so the live value ids the reorder walks are
        // sparse.
        state
            .release_value_pool_value(value_pool_id, U32Id::from_u32(4), U32Id::from_u32(0))
            .unwrap();
        state.validate().unwrap();

        order_palette_colors(&mut state, palette_id);

        // The three drawn colors lead in material order, then the unused one.
        assert_eq!(
            state.value_pool(value_pool_id),
            Some(
                &VoxValuePool::vec_4_float(vec![
                    [0.0, 0.0, 1.0, 1.0],
                    [1.0, 0.0, 0.0, 1.0],
                    [0.0, 1.0, 0.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                ])
                .unwrap()
            )
        );
        state.validate().unwrap();

        state.gc();
        state.validate().unwrap();
    }
}
