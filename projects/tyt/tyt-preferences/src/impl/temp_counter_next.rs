use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Returns the next value of a process-wide counter for temp-file names.
pub(crate) fn temp_counter_next() -> u64 {
    TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
}
