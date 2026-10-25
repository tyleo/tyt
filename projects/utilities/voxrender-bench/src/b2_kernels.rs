use crate::{
    BenchKernels, Error, FragmentLaunch, KernelWork, RayQueryArm, RayQueryTrace, Result, StepGrid,
    StepKernel, StepLoop, Walk, art_room_scene, grazing_floor_scene, subgroup_sizes,
};
use voxrender::RenderScene;
use wgpu::{Device, Queue};

/// The step counts each loop runs at. The report fits a line through them.
const COUNTS: [u32; 3] = [8, 16, 32];

/// The sides of the tree walk's regions: a cell, a 4^3 child, and a 16^3
/// tile.
const REGION_SIDES: [i32; 3] = [1, 4, 16];

/// The launch-only kernels' labels, in pass order.
const LAUNCHES: [&str; 2] = ["compute launch", "fragment launch"];

/// B2's kernels over a `width` by `height` eye, with the ray queries when
/// `ray_queries`.
pub fn b2_kernels(
    device: &Device,
    queue: &Queue,
    width: u32,
    height: u32,
    ray_queries: bool,
) -> Result<BenchKernels> {
    let subgroups = subgroup_sizes(device, queue, width, height)?;
    let grid = StepGrid::new(device, queue);
    let mut kernels = Vec::new();

    for step_loop in StepLoop::ALL {
        for count in COUNTS {
            kernels.push((
                format!("{}, {count} steps", step_loop.label()),
                KernelWork::Steps(StepKernel::new(
                    device, &grid, step_loop, width, height, count,
                )),
            ));
        }
    }

    kernels.push((
        LAUNCHES[0].to_owned(),
        KernelWork::Steps(StepKernel::new(
            device,
            &grid,
            StepLoop::Hoisted,
            width,
            height,
            0,
        )),
    ));
    kernels.push((
        LAUNCHES[1].to_owned(),
        KernelWork::Fragments(FragmentLaunch::new(device, width, height)),
    ));

    // Each trace's label and, for the tree walk, its mean loop iterations
    // per ray leaving each of `REGION_SIDES`.
    let mut traces = Vec::new();

    for (name, scene) in [
        ("S1 art room", art_room_scene()),
        ("S5 grazing floor", grazing_floor_scene()),
    ] {
        // B2 asks for the primary ray rate.
        let scene = unlit(scene);
        let (view_id, _) = scene.iter_views().next().expect("a bench scene has a view");

        let mut iterations = [0.0; REGION_SIDES.len()];

        for (mean, side) in iterations.iter_mut().zip(REGION_SIDES) {
            let counts = Walk::CountedTree { side }
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

            *mean = total as f64 / counts.len() as f64;
        }

        for (walk, label) in [
            (Walk::Tree, "tree walk"),
            (Walk::CheckedTree, "tree walk, checked"),
        ] {
            let label = format!("{label}, {name}");

            traces.push((label.clone(), Some(iterations)));
            kernels.push((
                label,
                KernelWork::Walk(walk.trace(device, &scene, view_id, width, height)?),
            ));
        }

        let arms: &[RayQueryArm] = if ray_queries { &RayQueryArm::ALL } else { &[] };

        for &arm in arms {
            let label = format!("{}, {name}", arm.label());
            let trace = RayQueryTrace::new(device, queue, &scene, view_id, width, height, arm)?;

            traces.push((label.clone(), None));
            kernels.push((label, KernelWork::RayQuery(trace)));
        }
    }

    Ok(BenchKernels {
        title: "B2 times",
        kernels,
        // Keeps a worn run short. Most kernels take longer than a frame.
        measured: 120,
        summary: Box::new(move |medians, rays| {
            let (steps, rest) = medians.split_at(StepLoop::ALL.len() * COUNTS.len());
            let (launches, trace_medians) = rest.split_at(LAUNCHES.len());

            let mut lines = vec![
                match subgroups {
                    Some((least, most)) => {
                        format!("Subgroup sizes of an 8x8 compute dispatch: {least} to {most}.")
                    }

                    None => "Subgroup sizes: the device lacks subgroups.".to_owned(),
                },
                String::new(),
            ];

            if !ray_queries {
                lines.push("The run leaves out the ray queries.".to_owned());
                lines.push(String::new());
            }

            lines.extend(step_lines(steps, rays));
            lines.push(String::new());
            lines.extend(launch_lines(launches, rays));
            lines.push(String::new());
            lines.extend(trace_lines(&traces, trace_medians, rays));

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

/// Each loop's least-squares line through its kernels' `medians` against the
/// step count, over `rays` rays a pass.
fn step_lines(medians: &[f64], rays: f64) -> Vec<String> {
    let counts = COUNTS.map(f64::from);
    let mean_count = counts.iter().sum::<f64>() / counts.len() as f64;

    let mut lines = vec![
        "| Loop | ms per step | Gsteps/s | ms at 0 steps |".to_owned(),
        "| --- | ---: | ---: | ---: |".to_owned(),
    ];

    for (step_loop, times) in StepLoop::ALL.iter().zip(medians.chunks(COUNTS.len())) {
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
            step_loop.label(),
            rays * f64::from(step_loop.chains()) / slope / 1e6,
            mean_time - slope * mean_count
        ));
    }

    lines
}

/// Each launch-only kernel's row from its `medians`, over `rays`
/// invocations a pass.
fn launch_lines(medians: &[f64], rays: f64) -> Vec<String> {
    let mut lines = vec![
        "| Launch | ms | Ginvocations/s |".to_owned(),
        "| --- | ---: | ---: |".to_owned(),
    ];

    for (label, median) in LAUNCHES.iter().zip(medians) {
        lines.push(format!(
            "| {label} | {median:.3} | {:.2} |",
            rays / median / 1e6
        ));
    }

    lines
}

/// Each trace's row from its `medians` and its mean iterations per ray in
/// `traces`, over `rays` rays a pass.
fn trace_lines(traces: &[(String, Option<[f64; 3]>)], medians: &[f64], rays: f64) -> Vec<String> {
    let mut lines = vec![
        "| Trace | Grays/s | Iterations per ray | Cell steps | 4^3 skips | 16^3 skips | Giterations/s |"
            .to_owned(),
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: |".to_owned(),
    ];

    for ((label, iterations), median) in traces.iter().zip(medians) {
        let rate = rays / median / 1e6;

        lines.push(match iterations {
            Some([cells, children, tiles]) => {
                let total = cells + children + tiles;

                format!(
                    "| {label} | {rate:.3} | {total:.1} | {cells:.1} | {children:.1} | {tiles:.1} | {:.2} |",
                    rate * total
                )
            }

            None => format!("| {label} | {rate:.3} | | | | | |"),
        });
    }

    lines
}
