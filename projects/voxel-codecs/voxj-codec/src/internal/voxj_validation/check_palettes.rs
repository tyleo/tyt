use crate::{Check, Failures};
use std::collections::HashSet;
use voxj::{VoxjMain, VoxjPalette, VoxjRuntimeState};

/// Per palette:
/// 1. every property has a non-empty name, distinct within the palette, and
///    an in-range value pool;
/// 2. row-major materials hold at least one row, one per material, each of
///    exactly one value-index per property, within the pool that property
///    binds.
pub fn check_palettes(main: &VoxjMain, failures: &mut Failures) {
    let state = &main.runtime_state;
    for (index, palette) in state.palettes.iter().enumerate() {
        if !failures.go() {
            return;
        }

        // One namespace per palette (rule 10.2).
        let mut seen = HashSet::with_capacity(palette.properties.len());

        for (position, property) in palette.properties.iter().enumerate() {
            check_name(index, position, &property.name, &mut seen, failures);
            if property.value_pool >= state.value_pools.len() {
                failures.report(
                    Check::Palettes,
                    format!(
                        "palette {index} property {position} references value pool {}, but the document has {} pools",
                        property.value_pool,
                        state.value_pools.len()
                    ),
                );
            }
            if !failures.go() {
                return;
            }
        }

        // Every palette is sampled, so it needs a material to sample (rule
        // 10.6).
        if palette.materials.is_empty() {
            failures.report(Check::Palettes, format!("palette {index} has no materials"));
            if !failures.go() {
                return;
            }
        }

        check_materials(index, palette, state, failures);
    }
}

/// A property's name is non-empty and not yet taken within the palette.
fn check_name<'a>(
    index: usize,
    position: usize,
    name: &'a str,
    seen: &mut HashSet<&'a str>,
    failures: &mut Failures,
) {
    if name.is_empty() {
        failures.report(
            Check::Palettes,
            format!("palette {index} property {position} has an empty name"),
        );
    } else if !seen.insert(name) {
        failures.report(
            Check::Palettes,
            format!("palette {index} lists property {name:?} more than once"),
        );
    }
}

/// Every materials row holds exactly one value-index per property, and
/// every value-index falls within the values of the pool its property names.
fn check_materials(
    index: usize,
    palette: &VoxjPalette,
    state: &VoxjRuntimeState,
    failures: &mut Failures,
) {
    let width = palette.properties.len();
    for (material, row) in palette.materials.iter().enumerate() {
        if row.len() != width {
            failures.report(
                Check::Palettes,
                format!(
                    "palette {index} material {material} has {} value-indices but the palette \
                     has {width} properties",
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
            let Some(pool) = state.value_pools.get(palette.properties[cell].value_pool) else {
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
