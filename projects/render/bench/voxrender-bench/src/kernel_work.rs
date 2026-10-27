use crate::{ChainKernel, FragmentLaunch, RayQueryTrace, StepKernel};
use voxrender_wgpu::HitTrace;
use wgpu::{
    CommandEncoder, ComputePass, ComputePassDescriptor, ComputePassTimestampWrites, QuerySet,
};

/// What one timed pass runs.
pub enum KernelWork {
    /// A walk over a view's rays.
    Walk(HitTrace),

    /// B1's dependent chain.
    Chain(ChainKernel),

    /// B2's fixed-step loop.
    Steps(StepKernel),

    /// A ray query over a view's rays.
    RayQuery(RayQueryTrace),

    /// A fragment launch over both eyes.
    Fragments(FragmentLaunch),
}

impl KernelWork {
    /// Records the work over both eyes into `encoder`, between the
    /// timestamps at `first` and `first + 1` of `query_set`.
    pub fn record(
        &self,
        encoder: &mut CommandEncoder,
        label: &str,
        query_set: &QuerySet,
        first: u32,
    ) {
        let timed = TimedPass {
            label,
            query_set,
            first,
        };

        match self {
            KernelWork::Walk(trace) => timed.dispatch_twice(encoder, |pass| trace.dispatch(pass)),
            KernelWork::Chain(chain) => timed.dispatch_twice(encoder, |pass| chain.dispatch(pass)),
            KernelWork::Steps(steps) => timed.dispatch_twice(encoder, |pass| steps.dispatch(pass)),
            KernelWork::RayQuery(trace) => {
                timed.dispatch_twice(encoder, |pass| trace.dispatch(pass))
            }
            KernelWork::Fragments(launch) => launch.record(encoder, query_set, first),
        }
    }
}

/// A compute pass's label and timestamps.
struct TimedPass<'a> {
    label: &'a str,

    query_set: &'a QuerySet,

    first: u32,
}

impl TimedPass<'_> {
    /// Records one pass into `encoder` that runs `dispatch` once per eye.
    fn dispatch_twice(
        &self,
        encoder: &mut CommandEncoder,
        dispatch: impl Fn(&mut ComputePass<'_>),
    ) {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some(self.label),
            timestamp_writes: Some(ComputePassTimestampWrites {
                query_set: self.query_set,
                beginning_of_pass_write_index: Some(self.first),
                end_of_pass_write_index: Some(self.first + 1),
            }),
        });

        for _ in 0..2 {
            dispatch(&mut pass);
        }
    }
}
