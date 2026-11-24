use std::thread;

/// The cores a load leaves to the thread that draws while it runs, and the
/// most threads it takes. Quest 3 gives an app three cores.
const SPARED: usize = 1;
const MOST: usize = 8;

/// How many threads a load's work runs on: the device's cores less
/// [`SPARED`], at least one and at most [`MOST`].
pub fn load_threads() -> usize {
    thread::available_parallelism()
        .map_or(1, |cores| cores.get().saturating_sub(SPARED))
        .clamp(1, MOST)
}
