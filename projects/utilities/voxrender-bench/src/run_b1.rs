use crate::{Error, Gpu, Result, reference_hits};
use voxrender::{
    RenderShadow,
    fixtures::{
        cube_scene, glass_scene, glow_scene, l_shape_scene, precision_shadow_scene,
        precision_view_scene, random_scene, room_scene, spot_room_scene, stairs_scene,
        two_placements_scene,
    },
};
use voxrender_wgpu::{GpuWalk, PixelHit, trace_hits};

/// B1's line: the largest share of a scene's samples whose ids may differ
/// from the reference's.
const LINE: f64 = 0.001;

/// The mismatches printed to stderr per scene and walk.
const SHOWN: usize = 3;

/// How one walk's samples compare with the reference's over a scene.
#[derive(Default)]
struct Tally {
    /// Pixels whose first hit differs.
    ids: usize,

    /// Corner shadow samples that differ on pixels whose first hit matches.
    shadows: usize,
}

/// Runs B1 on `gpu` over the golden and precision-sim scenes, each view
/// drawn over a `size` square image, and returns the report in Markdown.
/// The integer walk's primary ids and corner shadow bits are judged against
/// B1's line. The float walk's flips are counted for the record.
pub fn run_b1(gpu: &Gpu, size: u32) -> Result<String> {
    let scenes = [
        ("cube", cube_scene(RenderShadow::PerPixel)),
        ("glass", glass_scene(RenderShadow::PerPixel)),
        ("glow", glow_scene(RenderShadow::PerPixel)),
        ("l-shape", l_shape_scene(RenderShadow::PerPixel)),
        ("room", room_scene(RenderShadow::PerPixel)),
        ("spot-room", spot_room_scene(RenderShadow::PerPixel)),
        (
            "two-placements",
            two_placements_scene(RenderShadow::PerPixel),
        ),
        ("precision views", precision_view_scene()),
        (
            "precision shadows, sparse",
            precision_shadow_scene(random_scene(1, 0.15, 1)),
        ),
        (
            "precision shadows, dense",
            precision_shadow_scene(random_scene(2, 0.40, 1)),
        ),
        (
            "precision shadows, stairs",
            precision_shadow_scene(stairs_scene()),
        ),
    ];

    let info = &gpu.info;

    let mut lines = vec![
        "## B1 desktop: ids and corner shadows".to_owned(),
        String::new(),
        format!(
            "- Adapter: {} ({:?}){}",
            info.name,
            info.backend,
            if info.driver.is_empty() {
                String::new()
            } else {
                format!(", driver {} {}", info.driver, info.driver_info)
            }
        ),
        format!("- Image: {size} by {size} pixels per view"),
        format!(
            "- Line: the integer walk's ids differ on at most {}% of a scene's pixels \
             and of its corner shadow samples",
            LINE * 100.0
        ),
        String::new(),
        "| Scene | Views | Pixels | Shadow samples | Id mismatches | Shadow mismatches \
         | Float id flips | Float shadow flips |"
            .to_owned(),
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |".to_owned(),
    ];

    let mut pass = true;

    for (name, scene) in &scenes {
        let mut views = 0;
        let mut pixels = 0;
        let mut samples = 0;
        let mut integer = Tally::default();
        let mut float = Tally::default();

        for (view_id, _) in scene.iter_views() {
            let want = reference_hits(scene, view_id, size)?;

            views += 1;
            pixels += want.len();
            samples += want
                .iter()
                .flatten()
                .map(|hit| hit.lit.len() * 4)
                .sum::<usize>();

            for (walk, tally) in [
                (GpuWalk::Integer, &mut integer),
                (GpuWalk::Float, &mut float),
            ] {
                let got = trace_hits(&gpu.device, &gpu.queue, scene, view_id, size, walk)
                    .map_err(Error::Trace)?;

                for (pixel, (want, got)) in want.iter().zip(&got).enumerate() {
                    let shown = tally.ids + tally.shadows < SHOWN;
                    let (x, y) = (pixel as u32 % size, pixel as u32 / size);

                    if !same_id(want.as_ref(), got.as_ref()) {
                        tally.ids += 1;
                    } else if let (Some(want), Some(got)) = (want, got) {
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
                        eprintln!(
                            "{name}, {walk:?} walk, view {view_id}, pixel ({x}, {y}): \
                             reference {want:?}, GPU {got:?}"
                        );
                    }
                }
            }
        }

        let id_share = share(integer.ids, pixels);
        let shadow_share = share(integer.shadows, samples);
        pass &= id_share <= LINE && shadow_share <= LINE;

        lines.push(format!(
            "| {name} | {views} | {pixels} | {samples} | {} | {} | {} | {} |",
            count(integer.ids, pixels),
            count(integer.shadows, samples),
            count(float.ids, pixels),
            count(float.shadows, samples),
        ));
    }

    lines.push(String::new());
    lines.push(format!("Result: {}.", if pass { "pass" } else { "fail" }));
    lines.push(String::new());

    Ok(lines.join("\n"))
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
