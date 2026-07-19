use crate::{Check, Failures};
use std::collections::HashSet;
use voxj::{VoxjMain, VoxjPalette, VoxjRuntimeState};

/// Every palette's array properties have non-empty, distinct names and
/// in-range value pools, and its row-major materials hold one row per
/// material, each of exactly one value-index per array property, within the
/// pool that property binds.
pub fn check_palettes(main: &VoxjMain, failures: &mut Failures) {
    let state = &main.runtime_state;
    for (index, palette) in state.palettes.iter().enumerate() {
        if !failures.go() {
            return;
        }

        let mut seen = HashSet::with_capacity(palette.array_properties.len());
        for (position, property) in palette.array_properties.iter().enumerate() {
            if property.name.is_empty() {
                failures.report(
                    Check::Palettes,
                    format!("palette {index} array property {position} has an empty name"),
                );
            } else if !seen.insert(property.name.as_str()) {
                failures.report(
                    Check::Palettes,
                    format!(
                        "palette {index} lists property {:?} more than once",
                        property.name
                    ),
                );
            }
            if property.value_pool >= state.value_pools.len() {
                failures.report(
                    Check::Palettes,
                    format!(
                        "palette {index} array property {position} references value pool {}, but the document has {} pools",
                        property.value_pool,
                        state.value_pools.len()
                    ),
                );
            }
            if !failures.go() {
                return;
            }
        }

        check_materials(index, palette, state, failures);
    }
}

/// Every materials row holds exactly one value-index per array property, and
/// every value-index falls within the values of the pool its property names.
fn check_materials(
    index: usize,
    palette: &VoxjPalette,
    state: &VoxjRuntimeState,
    failures: &mut Failures,
) {
    let width = palette.array_properties.len();
    for (material, row) in palette.materials.iter().enumerate() {
        if row.len() != width {
            failures.report(
                Check::Palettes,
                format!(
                    "palette {index} material {material} has {} value-indices but the palette \
                     has {width} array properties",
                    row.len()
                ),
            );
            if !failures.go() {
                return;
            }
            // With the row misaligned, cells no longer pair with properties.
            continue;
        }

        for (cell, &value_index) in row.iter().enumerate() {
            // The row arity matches here, so the property always resolves; its
            // pool resolves only when the value pool is in range, already
            // reported above when it is not.
            let Some(pool) = state
                .value_pools
                .get(palette.array_properties[cell].value_pool)
            else {
                continue;
            };
            let pool_len = pool.values_len();
            if value_index >= pool_len {
                failures.report(
                    Check::Palettes,
                    format!(
                        "palette {index} material {material} value-index {value_index} \
                         is out of range for a pool with {pool_len} values"
                    ),
                );
                if !failures.go() {
                    return;
                }
            }
        }
    }
}
