use crate::{Check, Failures};
use std::collections::HashSet;
use voxj::{VoxjMain, VoxjPalette, VoxjRuntimeState};

/// Per palette:
/// 1. every property has a non-empty name, distinct within the palette, and
///    an in-range value pool;
/// 2. materials hold at least one row, one per material, each of exactly
///    one value-index per property, within the value pool that property binds.
pub fn check_palettes(main: &VoxjMain, failures: &mut Failures) {
    let state = &main.runtime_state;
    for (palette_index, palette) in state.palettes.iter().enumerate() {
        if !failures.go() {
            return;
        }

        // One namespace per palette (rule 10.2).
        let mut seen = HashSet::with_capacity(palette.properties.len());

        for (property_index, property) in palette.properties.iter().enumerate() {
            check_name(
                palette_index,
                property_index,
                &property.name,
                &mut seen,
                failures,
            );
            if property.value_pool >= state.value_pools.len() {
                failures.report(
                    Check::Palettes,
                    format!(
                        "palette {palette_index} property {property_index} references value pool {}, but the document has {} value pools",
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
        // 10.3).
        if palette.materials.is_empty() {
            failures.report(
                Check::Palettes,
                format!("palette {palette_index} has no materials"),
            );
            if !failures.go() {
                return;
            }
        }

        check_materials(palette_index, palette, state, failures);
    }
}

/// A property's name is non-empty and not yet taken within the palette.
fn check_name<'a>(
    palette_index: usize,
    property_index: usize,
    name: &'a str,
    seen: &mut HashSet<&'a str>,
    failures: &mut Failures,
) {
    if name.is_empty() {
        failures.report(
            Check::Palettes,
            format!("palette {palette_index} property {property_index} has an empty name"),
        );
    } else if !seen.insert(name) {
        failures.report(
            Check::Palettes,
            format!("palette {palette_index} lists property {name:?} more than once"),
        );
    }
}

/// Every materials row holds exactly one value-index per property, and every
/// value-index falls within the values of the value pool its property names.
fn check_materials(
    palette_index: usize,
    palette: &VoxjPalette,
    state: &VoxjRuntimeState,
    failures: &mut Failures,
) {
    let width = palette.properties.len();
    for (material_index, row) in palette.materials.iter().enumerate() {
        if row.len() != width {
            failures.report(
                Check::Palettes,
                format!(
                    "palette {palette_index} material {material_index} has {} value-indices but the \
                     palette has {width} properties",
                    row.len()
                ),
            );
            if !failures.go() {
                return;
            }
            // With the row misaligned, cells no longer pair with properties.
            continue;
        }

        for (property_index, &value_index) in row.iter().enumerate() {
            // The row arity matches here, so the property always resolves; its
            // value pool resolves only when the reference is in range, already
            // reported above when it is not.
            let Some(value_pool) = state
                .value_pools
                .get(palette.properties[property_index].value_pool)
            else {
                continue;
            };
            let value_pool_len = value_pool.values_len();
            if value_index >= value_pool_len {
                failures.report(
                    Check::Palettes,
                    format!(
                        "palette {palette_index} material {material_index} value-index {value_index} \
                         is out of range for a value pool with {value_pool_len} values"
                    ),
                );
                if !failures.go() {
                    return;
                }
            }
        }
    }
}
