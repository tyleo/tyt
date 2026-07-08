use crate::BASE_COLOR_FACTOR;
use branded_id::U32Id;
use voxcore::{BVoxPalette, VoxMain};

/// Reorders `palette`'s `baseColorFactor` colors into material id order: the
/// first material's color moves to index 0, the next new color to 1, and so
/// on, then any unused color. A voxelized palette has one color per material,
/// so its material color value-indices become 0, 1, 2, and up. A no-op without
/// a color binding; rendering is unchanged.
pub fn order_palette_colors(state: &mut VoxMain, palette: U32Id<BVoxPalette>) {
    let Some(palette_ref) = state.palette(palette) else {
        return;
    };
    let Some(color) = palette_ref.binding_by_attribute(BASE_COLOR_FACTOR) else {
        return;
    };
    let Some(pool) = palette_ref.binding(color).map(|binding| binding.pool) else {
        return;
    };
    let len = state.value_pool(pool).map_or(0, |pool| pool.values_len());

    // The used color value-indices in material id order, then the unused ones,
    // forming a permutation of the pool.
    let mut new_order: Vec<u32> = Vec::with_capacity(len);
    let mut seen = vec![false; len];
    for material in palette_ref.iter_materials() {
        let Some(index) = palette_ref.value_index(material, color) else {
            continue;
        };
        if let Some(slot) = seen.get_mut(index as usize).filter(|slot| !**slot) {
            *slot = true;
            new_order.push(index);
        }
    }
    for (index, used) in seen.iter().enumerate() {
        if !used {
            new_order.push(index as u32);
        }
    }

    state
        .reorder_value_pool(pool, &new_order)
        .expect("the color pool is live and new_order is a permutation by construction");
}

#[cfg(test)]
mod tests {
    use crate::{BASE_COLOR_FACTOR, order_palette_colors};
    use voxcore::{VoxMain, VoxPalette, VoxValuePool};

    #[test]
    fn orders_colors_to_material_id_order() {
        let mut state = VoxMain::default();
        // Three colors; materials reference them out of order: blue, red,
        // green.
        let pool = state.add_value_pool(VoxValuePool::Srgba {
            values: vec![
                [1.0, 0.0, 0.0, 1.0], // 0 red
                [0.0, 1.0, 0.0, 1.0], // 1 green
                [0.0, 0.0, 1.0, 1.0], // 2 blue
            ],
        });
        let mut palette = VoxPalette::default();
        let color = palette.add_binding(BASE_COLOR_FACTOR.to_owned(), pool);
        let blue = palette.add_material(vec![2]).unwrap();
        let red = palette.add_material(vec![0]).unwrap();
        let green = palette.add_material(vec![1]).unwrap();
        let palette_id = state.add_palette(palette);
        state.validate().unwrap();

        order_palette_colors(&mut state, palette_id);

        // The materials now reference colors in id order 0, 1, 2, and the pool
        // is reordered so each still resolves to its own color.
        let palette = state.palette(palette_id).unwrap();
        assert_eq!(palette.value_index(blue, color), Some(0));
        assert_eq!(palette.value_index(red, color), Some(1));
        assert_eq!(palette.value_index(green, color), Some(2));
        assert_eq!(
            state.value_pool(pool),
            Some(&VoxValuePool::Srgba {
                values: vec![
                    [0.0, 0.0, 1.0, 1.0],
                    [1.0, 0.0, 0.0, 1.0],
                    [0.0, 1.0, 0.0, 1.0],
                ],
            })
        );
        state.validate().unwrap();
    }
}
