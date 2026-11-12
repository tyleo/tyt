use crate::{
    BenchKernels, BoxMarch, BoxRaster, DepthProbe, DepthProbeMode, Error, FragmentLaunch,
    KernelWork, MeshRaster, RasterArm, RasterMesh, RayQueryArm, RayQueryTrace, Result, StepGrid,
    StepKernel, StepLoop, Walk, art_room_scene, city_scene, grazing_floor_scene, subgroup_sizes,
};
use voxrender::RenderScene;
use wgpu::{Device, Queue};

/// The step counts each loop runs at. The report fits a line through them.
const COUNTS: [u32; 3] = [8, 16, 32];

/// The sides of the tree walk's regions: a cell, a 4^3 child, and a 16^3
/// tile.
const REGION_SIDES: [i32; 3] = [1, 4, 16];

/// The step counts the compute launch's sweep runs the hoisted loop at. The
/// counts below the step loops' show where a launch's cost ends.
const LAUNCH_STEPS: [u32; 5] = [0, 1, 2, 4, 8];

/// The workgroup sizes the compute launch's sweep runs the hoisted loop at
/// with no step, beside [`StepKernel::GROUP`]. The sizes show whether a
/// launch's cost follows its groups or its invocations.
const LAUNCH_GROUPS: [[u32; 2]; 3] = [[4, 4], [16, 8], [16, 16]];

/// The fragment launch's label.
const FRAGMENT_LAUNCH: &str = "fragment launch";

/// B2's kernels over a `width` by `height` eye, with the ray queries when
/// `ray_queries`, with each raster arm the device has the features for, and
/// with the box rasters.
pub fn b2_kernels(
    device: &Device,
    queue: &Queue,
    width: u32,
    height: u32,
    ray_queries: bool,
) -> Result<BenchKernels> {
    let subgroups = subgroup_sizes(device, queue, width, height)?;

    let (raster_arms, absent_arms): (Vec<_>, Vec<_>) = RasterArm::ALL
        .into_iter()
        .partition(|arm| device.features().contains(arm.features()));
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

    for count in LAUNCH_STEPS {
        kernels.push((
            compute_launch(count),
            KernelWork::Steps(StepKernel::new(
                device,
                &grid,
                StepLoop::Hoisted,
                width,
                height,
                count,
            )),
        ));
    }

    for group in LAUNCH_GROUPS {
        kernels.push((
            grouped_launch(group),
            KernelWork::Steps(StepKernel::grouped(
                device,
                &grid,
                StepLoop::Hoisted,
                width,
                height,
                0,
                group,
            )),
        ));
    }

    kernels.push((
        FRAGMENT_LAUNCH.to_owned(),
        KernelWork::Fragments(FragmentLaunch::new(device, width, height)),
    ));

    // Each trace's label and, for the tree walk, its mean loop iterations
    // per ray leaving each of `REGION_SIDES`.
    let mut traces = Vec::new();

    // S2's 1025 placements take the walks and the ray queries seconds a
    // pass, so only the rasters trace it.
    for (name, scene, marches) in [
        ("S1 art room", art_room_scene(), true),
        ("S5 grazing floor", grazing_floor_scene(), true),
        ("S2 city", city_scene(), false),
    ] {
        // B2 asks for the primary ray rate.
        let scene = unlit(scene);
        let (view_id, _) = scene.iter_views().next().expect("a bench scene has a view");

        if marches {
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

        let mesh = RasterMesh::new(device, &scene)?;

        for &arm in &raster_arms {
            let label = format!("{}, {name}", arm.label());
            let raster = MeshRaster::new(device, &mesh, &scene, view_id, width, height, arm)?;

            traces.push((label.clone(), None));
            kernels.push((label, KernelWork::Raster(raster)));
        }

        for march in BoxMarch::ALL {
            let label = format!("{}, {name}", march.label());
            let raster = BoxRaster::new(device, &scene, view_id, width, height, march)?;

            traces.push((label.clone(), None));
            kernels.push((label, KernelWork::Boxes(raster)));
        }
    }

    let probe_modes: Vec<_> = DepthProbeMode::ALL
        .into_iter()
        .filter(|mode| device.features().contains(mode.features()))
        .collect();

    for &mode in &probe_modes {
        for (occluded, layers) in [(false, "open"), (true, "behind a near layer")] {
            kernels.push((
                format!("depth probe, {}, {layers}", mode.label()),
                KernelWork::Depth(DepthProbe::new(device, width, height, mode, occluded)?),
            ));
        }
    }

    Ok(BenchKernels {
        title: "B2 times",
        kernels,
        // Keeps a worn run short. Most kernels take longer than a frame.
        measured: 120,
        summary: Box::new(move |medians, rays| {
            let (steps, rest) = medians.split_at(StepLoop::ALL.len() * COUNTS.len());
            let (launches, rest) = rest.split_at(LAUNCH_STEPS.len() + LAUNCH_GROUPS.len() + 1);
            let (trace_medians, probes) = rest.split_at(traces.len());

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

            if !absent_arms.is_empty() {
                lines.push(format!(
                    "The device lacks the features of these raster arms: {}.",
                    absent_arms
                        .iter()
                        .map(|arm| arm.label())
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
                lines.push(String::new());
            }

            lines.extend(step_lines(steps, rays));
            lines.push(String::new());
            lines.extend(launch_lines(launches, rays));
            lines.push(String::new());
            lines.extend(trace_lines(&traces, trace_medians, rays));
            lines.push(String::new());
            lines.extend(probe_lines(&probe_modes, probes));

            lines
        }),
    })
}

/// The label of the compute launch's kernel at `count` steps.
fn compute_launch(count: u32) -> String {
    format!("compute launch, {count} steps")
}

fn grouped_launch([across, down]: [u32; 2]) -> String {
    format!("compute launch, 0 steps, {across}x{down} groups")
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

    let labels = LAUNCH_STEPS
        .map(compute_launch)
        .into_iter()
        .chain(LAUNCH_GROUPS.map(grouped_launch))
        .chain([FRAGMENT_LAUNCH.to_owned()]);

    for (label, median) in labels.zip(medians) {
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

/// Each depth probe's row from its `medians`: the mode's four costly layers
/// in the open, then behind a near layer.
fn probe_lines(modes: &[DepthProbeMode], medians: &[f64]) -> Vec<String> {
    let mut lines = vec![
        "| Depth probe | Open ms | Behind a near layer ms |".to_owned(),
        "| --- | ---: | ---: |".to_owned(),
    ];

    for (mode, times) in modes.iter().zip(medians.chunks(2)) {
        lines.push(format!(
            "| {} | {:.3} | {:.3} |",
            mode.label(),
            times[0],
            times[1]
        ));
    }

    lines
}
