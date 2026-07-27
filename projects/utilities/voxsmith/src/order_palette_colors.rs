use crate::BASE_COLOR_FACTOR;
use branded_id::U32Id;
use std::collections::HashSet;
use voxcore::{BVoxPalette, BVoxPoolValue, VoxMain};

/// Reorders `palette`'s `baseColorFactor` colors to material order: each
/// material's color in turn, then the colors no material uses. Rendering is
/// unchanged. A no-op without a `baseColorFactor` property.
///
/// Requires a referentially valid state, which
/// [`VoxMain::validate`](voxcore::VoxMain::validate) checks.
pub fn order_palette_colors(state: &mut VoxMain, palette: U32Id<BVoxPalette>) {
    let Some(palette_ref) = state.palette(palette) else {
        return;
    };
    let Some(color) = palette_ref.property_by_name(BASE_COLOR_FACTOR) else {
        return;
    };
    let Some(pool) = palette_ref.property(color).map(|property| property.pool) else {
        return;
    };
    let Some(pool_ref) = state.value_pool(pool) else {
        return;
    };

    // The used color value ids in material order, then the unused ones in
    // their current order, forming a permutation of the pool.
    let mut new_order: Vec<U32Id<BVoxPoolValue>> = Vec::with_capacity(pool_ref.values_len());
    let mut seen: HashSet<U32Id<BVoxPoolValue>> = HashSet::with_capacity(pool_ref.values_len());
    for material in palette_ref.iter_materials() {
        let Some(value_id) = palette_ref.value_id(material, color) else {
            continue;
        };
        if seen.insert(value_id) {
            new_order.push(value_id);
        }
    }
    for (value_id, _) in pool_ref.iter_values() {
        if !seen.contains(&value_id) {
            new_order.push(value_id);
        }
    }

    state
        .reorder_value_pool(pool, &new_order)
        .expect("the color pool is live and new_order is a permutation by construction");
}

#[cfg(test)]
mod tests {
    use crate::{BASE_COLOR_FACTOR, order_palette_colors};
    use branded_id::U32Id;
    use voxcore::{VoxMain, VoxPalette, VoxValuePool};

    #[test]
    fn orders_colors_to_material_order() {
        let mut state = VoxMain::default();
        // Three colors; materials reference them out of order: blue, red,
        // green.
        let pool = state.add_value_pool(
            VoxValuePool::srgba(vec![
                [1.0, 0.0, 0.0, 1.0], // 0 red
                [0.0, 1.0, 0.0, 1.0], // 1 green
                [0.0, 0.0, 1.0, 1.0], // 2 blue
            ])
            .unwrap(),
        );
        let mut palette = VoxPalette::default();
        let color = palette
            .add_property(BASE_COLOR_FACTOR.to_owned(), pool)
            .unwrap();
        let blue = palette.add_material(vec![U32Id::from_u32(2)]).unwrap();
        let red = palette.add_material(vec![U32Id::from_u32(0)]).unwrap();
        let green = palette.add_material(vec![U32Id::from_u32(1)]).unwrap();
        let palette_id = state.add_palette(palette).unwrap();
        state.validate().unwrap();

        order_palette_colors(&mut state, palette_id);

        // The pool lists the colors in material order. Ids and what each
        // material resolves to are unchanged.
        assert_eq!(
            state.value_pool(pool),
            Some(
                &VoxValuePool::srgba(vec![
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
        assert_eq!(palette.value_id(blue, color), Some(U32Id::from_u32(0)));
        assert_eq!(palette.value_id(red, color), Some(U32Id::from_u32(1)));
        assert_eq!(palette.value_id(green, color), Some(U32Id::from_u32(2)));
        state.validate().unwrap();
    }

    #[test]
    fn orders_colors_past_an_unused_color_and_a_hole() {
        let mut state = VoxMain::default();
        let pool = state.add_value_pool(
            VoxValuePool::srgba(vec![
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
            .add_property(BASE_COLOR_FACTOR.to_owned(), pool)
            .unwrap();
        palette.add_material(vec![U32Id::from_u32(2)]).unwrap();
        palette.add_material(vec![U32Id::from_u32(0)]).unwrap();
        palette.add_material(vec![U32Id::from_u32(1)]).unwrap();
        let palette_id = state.add_palette(palette).unwrap();

        // Hole the pool, so the live value ids the reorder walks are sparse.
        state
            .remove_pool_value(pool, U32Id::from_u32(4), U32Id::from_u32(0))
            .unwrap();
        state.validate().unwrap();

        order_palette_colors(&mut state, palette_id);

        // The three drawn colors lead in material order, then the unused one.
        assert_eq!(
            state.value_pool(pool),
            Some(
                &VoxValuePool::srgba(vec![
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
