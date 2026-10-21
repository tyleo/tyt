use crate::{
    BrickMarch, Error, Gpu, RayQueryArm, RayQueryTrace, Result, Walk, b1_scenes, reference_hits,
};
use branded_id::U32Id;
use voxrender::{BRenderView, RenderScene};
use voxrender_wgpu::PixelHit;

/// B1's line: the largest share of a scene's samples whose ids may differ
/// from the reference's.
const LINE: f64 = 0.001;

/// The mismatches the report lists per scene and walk.
const SHOWN: usize = 3;

/// A GPU trace B1 compares with the reference.
#[derive(Clone, Copy, Debug)]
enum Tracer {
    Walk(Walk),

    RayQuery(RayQueryArm),
}

/// How one trace's samples compare with the reference's over a scene.
#[derive(Default)]
struct Tally {
    /// Pixels whose first hit differs.
    ids: usize,

    /// Corner shadow samples that differ on pixels whose first hit matches.
    shadows: usize,
}

/// Runs B1 on `gpu` over its scenes, each view drawn over a `size` square
/// image, and returns the report in Markdown.
/// The integer and tree walks' primary ids and corner shadow bits, and the
/// integer ray-query marches' primary ids, are judged against B1's line. The
/// float walk's and float marches' flips are counted for the record. The
/// report lists the first mismatching pixels of each scene and trace.
pub fn run_b1(gpu: &Gpu, size: u32) -> Result<String> {
    let scenes = b1_scenes();

    let tracers: Vec<_> = [Walk::Integer, Walk::Float, Walk::Tree]
        .map(Tracer::Walk)
        .into_iter()
        .chain(RayQueryArm::ALL.map(Tracer::RayQuery))
        .collect();

    let info = &gpu.info;

    // Adreno's driver info spans several lines.
    let driver_info = info
        .driver_info
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(", ");

    let mut lines = vec![
        "## B1: ids and corner shadows".to_owned(),
        String::new(),
        format!(
            "- Adapter: {} ({:?}){}",
            info.name,
            info.backend,
            if info.driver.is_empty() {
                String::new()
            } else {
                format!(", driver {} {driver_info}", info.driver)
            }
        ),
        format!("- Image: {size} by {size} pixels per view"),
        format!(
            "- Line: the integer and tree walks' ids and shadows, and the integer \
             ray-query marches' ids, differ on at most {}% of a scene's samples",
            LINE * 100.0
        ),
        String::new(),
        format!(
            "| Scene | Views | Pixels | Shadow samples | Id mismatches | Shadow mismatches \
             | Float id flips | Float shadow flips | Tree id mismatches | Tree shadow mismatches \
             | {} |",
            RayQueryArm::ALL
                .map(|arm| format!(
                    "Ray query {}^3 {}",
                    arm.brick,
                    if judged(Tracer::RayQuery(arm)) {
                        "integer id mismatches"
                    } else {
                        "float id flips"
                    }
                ))
                .join(" | ")
        ),
        format!(
            "| --- | ---: | ---: | ---: | {}",
            "---: | ".repeat(6 + RayQueryArm::ALL.len())
        )
        .trim_end()
        .to_owned(),
    ];

    let mut pass = true;
    let mut mismatches = Vec::new();

    for (name, scene) in &scenes {
        let mut views = 0;
        let mut pixels = 0;
        let mut samples = 0;
        let mut tallies: Vec<_> = tracers.iter().map(|_| Tally::default()).collect();

        for (view_id, _) in scene.iter_views() {
            let want = reference_hits(scene, view_id, size)?;

            views += 1;
            pixels += want.len();
            samples += want
                .iter()
                .flatten()
                .map(|hit| hit.lit.len() * 4)
                .sum::<usize>();

            for (&tracer, tally) in tracers.iter().zip(&mut tallies) {
                let got = trace(gpu, tracer, scene, view_id, size)?;

                for (pixel, (want, got)) in want.iter().zip(&got).enumerate() {
                    let shown = tally.ids + tally.shadows < SHOWN;
                    let (x, y) = (pixel as u32 % size, pixel as u32 / size);

                    if !same_id(want.as_ref(), got.as_ref()) {
                        tally.ids += 1;
                    } else if let (Tracer::Walk(_), Some(want), Some(got)) = (tracer, want, got) {
                        tally.shadows += want
                            .lit
                            .iter()
                            .zip(&got.lit)
                            .map(|(want, got)| (want ^ got).count_ones() as usize)
                            .sum::<usize>();
                    } else {
                        continue;
                    }

                    if shown && want != got {
                        mismatches.push(format!(
                            "- {name}, {}, view {view_id}, pixel ({x}, {y}): \
                             reference {want:?}, GPU {got:?}",
                            label(tracer)
                        ));
                    }
                }
            }
        }

        for (&tracer, tally) in tracers.iter().zip(&tallies) {
            if judged(tracer) {
                pass &= share(tally.ids, pixels) <= LINE && share(tally.shadows, samples) <= LINE;
            }
        }

        let columns: Vec<_> = tracers
            .iter()
            .zip(&tallies)
            .flat_map(|(tracer, tally)| match tracer {
                Tracer::Walk(_) => vec![count(tally.ids, pixels), count(tally.shadows, samples)],
                Tracer::RayQuery(_) => vec![count(tally.ids, pixels)],
            })
            .collect();

        lines.push(format!(
            "| {name} | {views} | {pixels} | {samples} | {} |",
            columns.join(" | ")
        ));
    }

    lines.push(String::new());
    lines.push(format!("Result: {}.", if pass { "pass" } else { "fail" }));

    if !mismatches.is_empty() {
        lines.push(String::new());
        lines.push("The first mismatching pixels of each scene and trace:".to_owned());
        lines.push(String::new());
        lines.extend(mismatches);
    }

    lines.push(String::new());

    Ok(lines.join("\n"))
}

/// `tracer`'s first hits of `view_id` over a `size` square image.
fn trace(
    gpu: &Gpu,
    tracer: Tracer,
    scene: &RenderScene,
    view_id: U32Id<BRenderView>,
    size: u32,
) -> Result<Vec<Option<PixelHit>>> {
    match tracer {
        Tracer::Walk(walk) => walk
            .trace(&gpu.device, scene, view_id, size, size)?
            .trace(&gpu.device, &gpu.queue)
            .map_err(Error::Trace),

        Tracer::RayQuery(arm) => {
            RayQueryTrace::new(&gpu.device, &gpu.queue, scene, view_id, size, size, arm)?
                .trace(&gpu.device, &gpu.queue)
        }
    }
}

/// Whether B1's line judges `tracer`.
fn judged(tracer: Tracer) -> bool {
    !matches!(
        tracer,
        Tracer::Walk(Walk::Float)
            | Tracer::RayQuery(RayQueryArm {
                march: BrickMarch::Float,
                ..
            })
    )
}

/// `tracer`'s name in the list of mismatches.
fn label(tracer: Tracer) -> String {
    match tracer {
        Tracer::Walk(walk) => format!("{walk:?} walk"),
        Tracer::RayQuery(arm) => arm.label(),
    }
}

/// Whether two pixels hit the same placement, cell, and face.
fn same_id(want: Option<&PixelHit>, got: Option<&PixelHit>) -> bool {
    match (want, got) {
        (Some(want), Some(got)) => {
            (want.placement, want.voxel, want.face) == (got.placement, got.voxel, got.face)
        }

        (None, None) => true,

        _ => false,
    }
}

/// `part` over `whole`, or zero when there is no whole.
fn share(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        0.0
    } else {
        part as f64 / whole as f64
    }
}

/// `part` and its share of `whole`.
fn count(part: usize, whole: usize) -> String {
    format!("{part} ({:.3}%)", share(part, whole) * 100.0)
}
