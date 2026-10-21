use crate::KernelWork;

/// A timed benchmark's setup for [`PassTimer`](crate::PassTimer).
pub struct BenchKernels {
    /// The report's heading.
    pub title: &'static str,

    /// Each kernel's label and work, in pass order.
    pub kernels: Vec<(String, KernelWork)>,

    /// The closing lines from each kernel's p50 in milliseconds and the
    /// rays in a pass.
    pub summary: fn(&[f64], f64) -> Vec<String>,
}
