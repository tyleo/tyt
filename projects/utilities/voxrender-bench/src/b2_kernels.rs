use crate::{BenchKernels, KernelWork, StepGrid, StepKernel, StepLayout};
use wgpu::{Device, Queue};

/// The step counts each layout runs at. The report fits a line through
/// them.
const COUNTS: [u32; 3] = [8, 16, 32];

/// B2's kernels over a `width` by `height` eye.
pub fn b2_kernels(device: &Device, queue: &Queue, width: u32, height: u32) -> BenchKernels {
    let grid = StepGrid::new(device, queue);
    let mut kernels = Vec::new();

    for layout in StepLayout::ALL {
        for count in COUNTS {
            kernels.push((
                format!("{}, {count} steps", layout.label()),
                KernelWork::Steps(StepKernel::new(device, &grid, layout, width, height, count)),
            ));
        }
    }

    BenchKernels {
        title: "B2 step times",
        kernels,
        summary,
    }
}

/// Each layout's least-squares line through its kernels' `medians` against
/// the step count, over `rays` rays a pass.
fn summary(medians: &[f64], rays: f64) -> Vec<String> {
    let counts = COUNTS.map(f64::from);
    let mean_count = counts.iter().sum::<f64>() / counts.len() as f64;

    let mut lines = vec![
        "| Layout | ms per step | Gsteps/s | ms at 0 steps |".to_owned(),
        "| --- | ---: | ---: | ---: |".to_owned(),
    ];

    for (layout, times) in StepLayout::ALL.iter().zip(medians.chunks(COUNTS.len())) {
        let mean_time = times.iter().sum::<f64>() / times.len() as f64;

        let (moment, spread) =
            counts
                .iter()
                .zip(times)
                .fold((0.0, 0.0), |(moment, spread), (count, time)| {
                    let offset = count - mean_count;

                    (
                        moment + offset * (time - mean_time),
                        spread + offset * offset,
                    )
                });

        let slope = moment / spread;

        lines.push(format!(
            "| {} | {slope:.4} | {:.2} | {:.3} |",
            layout.label(),
            rays / slope / 1e6,
            mean_time - slope * mean_count
        ));
    }

    lines
}
