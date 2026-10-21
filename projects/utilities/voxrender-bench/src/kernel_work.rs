use crate::{ChainKernel, StepKernel};
use voxrender_wgpu::HitTrace;
use wgpu::ComputePass;

/// What one timed pass runs.
pub enum KernelWork {
    /// A walk over a view's rays.
    Walk(HitTrace),

    /// B1's dependent chain.
    Chain(ChainKernel),

    /// B2's fixed-step loop.
    Steps(StepKernel),
}

impl KernelWork {
    /// Records the work over one eye into `pass`.
    pub fn dispatch(&self, pass: &mut ComputePass<'_>) {
        match self {
            KernelWork::Walk(trace) => trace.dispatch(pass),
            KernelWork::Chain(chain) => chain.dispatch(pass),
            KernelWork::Steps(steps) => steps.dispatch(pass),
        }
    }
}
