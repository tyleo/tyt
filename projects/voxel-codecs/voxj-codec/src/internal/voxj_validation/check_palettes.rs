use crate::{Check, Failures};
use std::collections::HashSet;
use voxj::{VoxjMain, VoxjPalette, VoxjRuntimeState};

/// Every palette has non-empty bindings with distinct attributes and in-range
/// pool refs, and column-major materials with one column per binding, a shared
/// length M at least 1, and value-indices within each bound pool.
pub fn check_palettes(main: &VoxjMain, failures: &mut Failures) {
    let state = &main.runtime_state;
    for (index, palette) in state.palettes.iter().enumerate() {
        if !failures.go() {
            return;
        }

        if palette.bindings.is_empty() {
            failures.report(Check::Palettes, format!("palette {index} has no bindings"));
            if !failures.go() {
                return;
            }
        }

        let mut seen = HashSet::with_capacity(palette.bindings.len());
        for (column, binding) in palette.bindings.iter().enumerate() {
            if binding.attribute.is_empty() {
                failures.report(
                    Check::Palettes,
                    format!("palette {index} binding {column} has an empty attribute name"),
                );
            } else if !seen.insert(binding.attribute.as_str()) {
                failures.report(
                    Check::Palettes,
                    format!(
                        "palette {index} binds attribute {:?} more than once",
                        binding.attribute
                    ),
                );
            }
            if binding.pool_ref >= state.value_pools.len() {
                failures.report(
                    Check::Palettes,
                    format!(
                        "palette {index} binding {column} references value pool {}, but the document has {} pools",
                        binding.pool_ref,
                        state.value_pools.len()
                    ),
                );
            }
            if !failures.go() {
                return;
            }
        }

        // Materials must be one column per binding before the columns can be
        // aligned to their pools; a mismatch makes the per-column checks moot.
        if palette.materials.len() != palette.bindings.len() {
            failures.report(
                Check::Palettes,
                format!(
                    "palette {index} has {} materials columns but {} bindings",
                    palette.materials.len(),
                    palette.bindings.len()
                ),
            );
            continue;
        }

        check_materials(index, palette, state, failures);
    }
}

/// Every materials column shares the length M at least 1, and every value-index
/// falls within the values of the pool its binding names.
fn check_materials(
    index: usize,
    palette: &VoxjPalette,
    state: &VoxjRuntimeState,
    failures: &mut Failures,
) {
    let m = palette.materials.first().map_or(0, Vec::len);
    if !palette.materials.is_empty() && m == 0 {
        failures.report(
            Check::Palettes,
            format!("palette {index} has no materials, so M must be at least 1"),
        );
        if !failures.go() {
            return;
        }
    }

    for (column, values) in palette.materials.iter().enumerate() {
        if values.len() != m {
            failures.report(
                Check::Palettes,
                format!(
                    "palette {index} materials column {column} has {} entries but column 0 has {m}",
                    values.len()
                ),
            );
            if !failures.go() {
                return;
            }
        }

        // The column/binding counts match here, so the binding always resolves;
        // its pool resolves only when the pool ref is in range, already reported
        // above when it is not.
        let Some(pool) = palette
            .bindings
            .get(column)
            .and_then(|binding| state.value_pools.get(binding.pool_ref))
        else {
            continue;
        };
        let pool_len = pool.values_len();
        for &value_index in values {
            if value_index >= pool_len {
                failures.report(
                    Check::Palettes,
                    format!(
                        "palette {index} materials column {column} value-index {value_index} \
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
