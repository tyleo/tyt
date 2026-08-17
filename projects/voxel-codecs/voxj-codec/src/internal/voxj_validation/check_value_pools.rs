use crate::{Check, Failures};
use voxj::VoxjMain;

/// Every value pool has non-empty `values`, the one clause of spec rule 9 the
/// typed value-pool model cannot enforce: an empty `values` is a well-typed
/// `Vec`.
pub fn check_value_pools(main: &VoxjMain, failures: &mut Failures) {
    for (value_pool_index, value_pool) in main.runtime_state.value_pools.iter().enumerate() {
        if !failures.go() {
            return;
        }

        if value_pool.is_empty() {
            failures.report(
                Check::ValuePools,
                format!("value pool {value_pool_index} has no values"),
            );
        }
    }
}
