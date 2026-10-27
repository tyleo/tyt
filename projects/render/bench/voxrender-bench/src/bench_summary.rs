/// A timed benchmark's closing lines from each kernel's p50 in milliseconds
/// and the rays in a pass.
pub type BenchSummary = Box<dyn Fn(&[f64], f64) -> Vec<String>>;
