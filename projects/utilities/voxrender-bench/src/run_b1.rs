use crate::{Error, Gpu, Result, Walk, b1_scenes, reference_hits};
use voxrender_wgpu::PixelHit;

/// B1's line: the largest share of a scene's samples whose ids may differ
/// from the reference's.
const LINE: f64 = 0.001;

/// The mismatches the report lists per scene and walk.
const SHOWN: usize = 3;

/// How one walk's samples compare with the reference's over a scene.
#[derive(Default)]
struct Tally {
    /// Pixels whose first hit differs.
    ids: usize,

    /// Corner shadow samples that differ on pixels whose first hit matches.
    shadows: usize,
}

/// Runs B1 on `gpu` over its scenes, each view drawn over a `size` square
/// image, and returns the report in Markdown.
/// The integer and tree walks' primary ids and corner shadow bits are judged
/// against B1's line. The float walk's flips are counted for the record. The
/// report lists the first mismatching pixels of each scene and walk.
pub fn run_b1(gpu: &Gpu, size: u32) -> Result<String> {
    let scenes = b1_scenes();

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
            "- Line: the integer and tree walks' ids differ on at most {}% of a scene's pixels \
             and of its corner shadow samples",
            LINE * 100.0
        ),
        String::new(),
        "| Scene | Views | Pixels | Shadow samples | Id mismatches | Shadow mismatches \
         | Float id flips | Float shadow flips | Tree id mismatches | Tree shadow mismatches |"
            .to_owned(),
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |".to_owned(),
    ];

    let mut pass = true;
    let mut mismatches = Vec::new();

    for (name, scene) in &scenes {
        let mut views = 0;
        let mut pixels = 0;
        let mut samples = 0;
        let mut integer = Tally::default();
        let mut float = Tally::default();
        let mut tree = Tally::default();

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
                (Walk::Integer, &mut integer),
                (Walk::Float, &mut float),
                (Walk::Tree, &mut tree),
            ] {
                let got = walk
                    .trace(&gpu.device, scene, view_id, size, size)?
                    .trace(&gpu.device, &gpu.queue)
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
                        mismatches.push(format!(
                            "- {name}, {walk:?} walk, view {view_id}, pixel ({x}, {y}): \
                             reference {want:?}, GPU {got:?}"
                        ));
                    }
                }
            }
        }

        for tally in [&integer, &tree] {
            pass &= share(tally.ids, pixels) <= LINE && share(tally.shadows, samples) <= LINE;
        }

        lines.push(format!(
            "| {name} | {views} | {pixels} | {samples} | {} | {} | {} | {} | {} | {} |",
            count(integer.ids, pixels),
            count(integer.shadows, samples),
            count(float.ids, pixels),
            count(float.shadows, samples),
            count(tree.ids, pixels),
            count(tree.shadows, samples),
        ));
    }

    lines.push(String::new());
    lines.push(format!("Result: {}.", if pass { "pass" } else { "fail" }));

    if !mismatches.is_empty() {
        lines.push(String::new());
        lines.push("The first mismatching pixels of each scene and walk:".to_owned());
        lines.push(String::new());
        lines.extend(mismatches);
    }

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
