use crate::{Check, Failures};
use std::collections::HashSet;
use voxj::{VoxjMain, VoxjPalette, VoxjRuntimeState};

/// Every palette's properties have non-empty names, distinct across its array
/// and scalar properties together, and in-range value pools; its scalar
/// properties pin in-range value-indices; and its row-major materials hold one
/// row per material, each of exactly one value-index per array property,
/// within the pool that property binds.
pub fn check_palettes(main: &VoxjMain, failures: &mut Failures) {
    let state = &main.runtime_state;
    for (index, palette) in state.palettes.iter().enumerate() {
        if !failures.go() {
            return;
        }

        // One namespace across both property lists (rule 10.2).
        let mut seen = HashSet::with_capacity(
            palette.array_properties.len() + palette.scalar_properties.len(),
        );

        for (position, property) in palette.array_properties.iter().enumerate() {
            check_name(
                index,
                "array",
                position,
                &property.name,
                &mut seen,
                failures,
            );
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

        for (position, property) in palette.scalar_properties.iter().enumerate() {
            check_name(
                index,
                "scalar",
                position,
                &property.name,
                &mut seen,
                failures,
            );
            // The pinned cell gets the same range checks as a materials cell:
            // the pool resolves and the value-index lands inside it.
            match state.value_pools.get(property.value_pool) {
                None => {
                    failures.report(
                        Check::Palettes,
                        format!(
                            "palette {index} scalar property {position} references value pool {}, but the document has {} pools",
                            property.value_pool,
                            state.value_pools.len()
                        ),
                    );
                }
                Some(pool) => {
                    let pool_len = pool.values_len();
                    if property.value_index >= pool_len {
                        failures.report(
                            Check::Palettes,
                            format!(
                                "palette {index} scalar property {position} value-index {} \
                                 is out of range for a pool with {pool_len} values",
                                property.value_index
                            ),
                        );
                    }
                }
            }
            if !failures.go() {
                return;
            }
        }

        check_materials(index, palette, state, failures);
    }
}

/// A property's name is non-empty and not yet taken within the palette; `kind`
/// names the list for the message.
fn check_name<'a>(
    index: usize,
    kind: &str,
    position: usize,
    name: &'a str,
    seen: &mut HashSet<&'a str>,
    failures: &mut Failures,
) {
    if name.is_empty() {
        failures.report(
            Check::Palettes,
            format!("palette {index} {kind} property {position} has an empty name"),
        );
    } else if !seen.insert(name) {
        failures.report(
            Check::Palettes,
            format!("palette {index} lists property {name:?} more than once"),
        );
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
