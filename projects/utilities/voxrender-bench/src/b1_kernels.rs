use crate::{BenchKernels, ChainKernel, KernelWork, Result, Walk, far_wall_scene};
use wgpu::Device;

/// The far wall's depths in cells. The walks' growth between them is the
/// steps' cost alone.
const DEPTHS: [u32; 2] = [8, 16];

/// B1's kernels over a `width` by `height` eye.
pub fn b1_kernels(device: &Device, width: u32, height: u32) -> Result<BenchKernels> {
    let mut kernels = Vec::new();

    for depth in DEPTHS {
        let scene = far_wall_scene(depth);
        let (view_id, _) = scene
            .iter_views()
            .next()
            .expect("the far wall scene has a view");

        for (name, walk) in [("integer", Walk::Integer), ("float", Walk::Float)] {
            let trace = walk.trace(device, &scene, view_id, width, height)?;

            kernels.push((
                format!("{name} walk, {depth} deep"),
                KernelWork::Walk(trace),
            ));
        }
    }

    for (name, float) in [("i32", false), ("f32", true)] {
        kernels.push((
            format!("{name} chain"),
            KernelWork::Chain(ChainKernel::new(device, width, height, float)),
        ));
    }

    Ok(BenchKernels {
        title: "B1 times",
        kernels,
        measured: 600,
        summary: Box::new(summary),
    })
}

/// The step and chain ratios from the kernels' `medians`.
fn summary(medians: &[f64], _rays: f64) -> Vec<String> {
    let [near, far] = DEPTHS;

    vec![format!(
        "Integer step over float step, from the walks' growth between {near} and {far} \
         cells deep: {:.3}x. i32 chain over f32 chain: {:.3}x.",
        (medians[2] - medians[0]) / (medians[3] - medians[1]),
        medians[4] / medians[5]
    )]
}
