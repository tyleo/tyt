/// A benchmark that [`PassTimer`](crate::PassTimer) times.
#[derive(Clone, Copy, Debug)]
pub enum TimedBench {
    /// B1's walks over the far wall and its dependent chains.
    B1,

    /// B2's fixed-step loops and launch-only kernels, and its tree walks and
    /// ray queries over S1 and S5.
    B2 {
        /// Whether the run times the ray queries. A Metal capture hides them
        /// from wgpu.
        ray_queries: bool,
    },
}
