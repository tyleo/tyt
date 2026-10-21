/// A benchmark that [`PassTimer`](crate::PassTimer) times.
#[derive(Clone, Copy, Debug)]
pub enum TimedBench {
    /// B1's walks over the far wall and its dependent chains.
    B1,

    /// B2's fixed-step loops over each memory layout.
    B2,
}
