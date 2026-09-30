use crate::{RenderOcclusion, RenderOutput, RenderScene, RenderShadow, render};
use png::{BitDepth, ColorType, Decoder, Encoder};
use std::{env, fs, io::Cursor, path::Path};

/// The environment variable that rewrites the goldens instead of checking
/// them.
const UPDATE_GOLDENS: &str = "VOXRENDER_UPDATE_GOLDENS";

/// The side of every golden image in pixels.
const SIZE: u32 = 64;

/// How far a channel may stray from the golden's.
const TOLERANCE: u8 = 2;

/// Checks `fixture`'s render under each variant against its golden.
///
/// # Arguments
/// * `name` - prefixes the golden file names.
/// * `goldens` - one per variant, in the order `per-pixel`, `per-face`,
///   `per-corner`, and `unoccluded`.
pub fn check_goldens(name: &str, fixture: fn(RenderShadow) -> RenderScene, goldens: [&[u8]; 4]) {
    let variants = [
        ("per-pixel", RenderShadow::PerPixel, RenderOcclusion::Corner),
        ("per-face", RenderShadow::PerFace, RenderOcclusion::Corner),
        (
            "per-corner",
            RenderShadow::PerCorner,
            RenderOcclusion::Corner,
        ),
        ("unoccluded", RenderShadow::PerPixel, RenderOcclusion::None),
    ];

    for ((variant, shadow, occlusion), golden) in variants.into_iter().zip(goldens) {
        let scene = fixture(shadow);

        let (view_id, _) = scene.iter_views().next().expect("a fixture holds one view");

        let image = render(&scene, view_id, occlusion, SIZE, SIZE).unwrap();

        let output = RenderOutput::from_image(&image, None);

        let file = format!("{name}-{variant}");

        if env::var_os(UPDATE_GOLDENS).is_some() {
            write_golden(&file, &output);

            continue;
        }

        compare(&file, golden, &output);
    }
}

fn write_golden(file: &str, output: &RenderOutput) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/goldens")
        .join(format!("{file}.png"));

    let mut bytes = Vec::new();

    let mut encoder = Encoder::new(&mut bytes, output.width(), output.height());
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&output.to_bytes()).unwrap();
    writer.finish().unwrap();

    fs::write(&path, bytes).unwrap();
}

/// Panics unless `output` matches the decoded `golden` per channel within
/// `TOLERANCE`.
fn compare(file: &str, golden: &[u8], output: &RenderOutput) {
    assert!(
        !golden.is_empty(),
        "no golden for {file}; run with {UPDATE_GOLDENS}=1 to write it"
    );

    let mut reader = Decoder::new(Cursor::new(golden)).read_info().unwrap();
    let mut buffer = vec![0u8; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buffer).unwrap();
    buffer.truncate(info.buffer_size());

    assert_eq!(info.color_type, ColorType::Rgba, "{file}");
    assert_eq!(
        (info.width, info.height),
        (output.width(), output.height()),
        "{file}"
    );

    let rendered = output.to_bytes();

    let mismatches: Vec<(usize, u8, u8)> = buffer
        .iter()
        .zip(&rendered)
        .enumerate()
        .filter(|(_, (expected, actual))| expected.abs_diff(**actual) > TOLERANCE)
        .map(|(index, (expected, actual))| (index, *expected, *actual))
        .collect();

    assert!(
        mismatches.is_empty(),
        "{file}: {} channels stray past {TOLERANCE}, the first at byte {} ({} in the golden, {} \
         rendered); run with {UPDATE_GOLDENS}=1 to accept the render",
        mismatches.len(),
        mismatches[0].0,
        mismatches[0].1,
        mismatches[0].2
    );
}
