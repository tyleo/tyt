use crate::{Gpu, Result, gpu_hits, reference_hits};
use voxrender::{
    RenderShadow,
    fixtures::{
        cube_scene, glass_scene, glow_scene, l_shape_scene, precision_view_scene, room_scene,
        spot_room_scene, two_placements_scene,
    },
};

/// B1's line: the largest share of a scene's samples whose ids may differ
/// from the reference's.
const LINE: f64 = 0.001;

/// The mismatches printed to stderr per scene.
const SHOWN: usize = 3;

/// Runs B1's desktop half for primary ids over the golden and precision-sim
/// scenes, each view drawn over a `size` square image, and returns the
/// report in Markdown.
pub fn run_b1(size: u32) -> Result<String> {
    let gpu = Gpu::new()?;

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
    ];

    let info = &gpu.info;

    let mut lines = vec![
        "## B1 desktop: primary ids".to_owned(),
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
            "- Line: id mismatches on at most {}% of samples per scene",
            LINE * 100.0
        ),
        String::new(),
        "| Scene | Views | Samples | Hits | Mismatches | Share |".to_owned(),
        "| --- | ---: | ---: | ---: | ---: | ---: |".to_owned(),
    ];

    let mut pass = true;

    for (name, scene) in &scenes {
        let mut views = 0;
        let mut samples = 0;
        let mut hits = 0;
        let mut mismatches = 0;

        for (view_id, _) in scene.iter_views() {
            let want = reference_hits(scene, view_id, size)?;
            let got = gpu_hits(&gpu, scene, view_id, size)?;

            views += 1;
            samples += want.len();
            hits += want.iter().flatten().count();

            for (pixel, (want, got)) in want.iter().zip(&got).enumerate() {
                if want == got {
                    continue;
                }

                if mismatches < SHOWN {
                    let (x, y) = (pixel as u32 % size, pixel as u32 / size);

                    eprintln!(
                        "{name}, view {view_id}, pixel ({x}, {y}): reference {want:?}, GPU {got:?}"
                    );
                }

                mismatches += 1;
            }
        }

        let share = mismatches as f64 / samples as f64;
        pass &= share <= LINE;

        lines.push(format!(
            "| {name} | {views} | {samples} | {hits} | {mismatches} | {:.3}% |",
            share * 100.0
        ));
    }

    lines.push(String::new());
    lines.push(format!("Result: {}.", if pass { "pass" } else { "fail" }));
    lines.push(String::new());

    Ok(lines.join("\n"))
}
