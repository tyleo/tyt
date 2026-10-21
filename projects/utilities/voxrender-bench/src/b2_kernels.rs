use crate::{
    BenchKernels, Error, KernelWork, Result, StepGrid, StepKernel, StepLayout, Walk,
    art_room_scene, grazing_floor_scene,
};
use voxrender::RenderScene;
use wgpu::{Device, Queue};

/// The step counts each layout runs at. The report fits a line through
/// them.
const COUNTS: [u32; 3] = [8, 16, 32];

/// B2's kernels over a `width` by `height` eye.
pub fn b2_kernels(device: &Device, queue: &Queue, width: u32, height: u32) -> Result<BenchKernels> {
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

    // Each scene's mean loop iterations per ray.
    let mut walks = Vec::new();

    for (name, scene) in [
        ("S1 art room", art_room_scene()),
        ("S5 grazing floor", grazing_floor_scene()),
    ] {
        // B2 asks for the primary ray rate.
        let scene = unlit(scene);
        let (view_id, _) = scene.iter_views().next().expect("a bench scene has a view");

        let counts = Walk::CountedTree
            .trace(device, &scene, view_id, width, height)?
            .trace(device, queue)
            .map_err(Error::Trace)?;

        let total: u64 = counts
            .iter()
            .map(|hit| {
                u64::from(
                    hit.as_ref()
                        .expect("the counted walk hits every pixel")
                        .voxel,
                )
            })
            .sum();

        walks.push((name, total as f64 / counts.len() as f64));

        kernels.push((
            format!("tree walk, {name}"),
            KernelWork::Walk(Walk::Tree.trace(device, &scene, view_id, width, height)?),
        ));
    }

    Ok(BenchKernels {
        title: "B2 times",
        kernels,
        summary: Box::new(move |medians, rays| {
            let (steps, walk_medians) = medians.split_at(StepLayout::ALL.len() * COUNTS.len());
            let mut lines = step_lines(steps, rays);

            lines.push(String::new());
            lines.extend(walk_lines(&walks, walk_medians, rays));

            lines
        }),
    })
}

/// `scene` without its lights.
fn unlit(mut scene: RenderScene) -> RenderScene {
    let light_ids: Vec<_> = scene.iter_lights().map(|(light_id, _)| light_id).collect();

    for light_id in light_ids {
        scene
            .release_light(light_id)
            .expect("the light is the scene's");
    }

    scene
}

/// Each layout's least-squares line through its kernels' `medians` against
/// the step count, over `rays` rays a pass.
fn step_lines(medians: &[f64], rays: f64) -> Vec<String> {
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

/// Each scene's row from its walk's `medians` and its mean iterations per
/// ray in `walks`, over `rays` rays a pass.
fn walk_lines(walks: &[(&str, f64)], medians: &[f64], rays: f64) -> Vec<String> {
    let mut lines = vec![
        "| Scene | Grays/s | Iterations per ray | Giterations/s |".to_owned(),
        "| --- | ---: | ---: | ---: |".to_owned(),
    ];

    for ((name, iterations), median) in walks.iter().zip(medians) {
        let rate = rays / median / 1e6;

        lines.push(format!(
            "| {name} | {rate:.3} | {iterations:.1} | {:.2} |",
            rate * iterations
        ));
    }

    lines
}
