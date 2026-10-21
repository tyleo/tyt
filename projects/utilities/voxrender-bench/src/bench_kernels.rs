use crate::{BenchSummary, KernelWork};

/// A timed benchmark's setup for [`PassTimer`](crate::PassTimer).
pub struct BenchKernels {
    /// The report's heading.
    pub title: &'static str,

    /// Each kernel's label and work, in pass order.
    pub kernels: Vec<(String, KernelWork)>,

    /// The report's closing lines.
    pub summary: BenchSummary,
}
