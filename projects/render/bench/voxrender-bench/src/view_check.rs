use crate::{BoxView, Error, Gpu, Result, TileScene, view_image};
use branded_id::U32Id;
use ty_math::{TyLinSrgbF32, TySrgbU8};
use voxrender::{BRenderView, RenderBloom, RenderOcclusion, RenderOutput, RenderScene};
use voxrender_cpu::{MapRows, render};

/// The 8-bit steps a channel can differ by before its pixel counts as off.
const TOLERANCE: u8 = 8;

/// Maps a render's rows one after another.
struct SerialRows;

impl MapRows for SerialRows {
    fn map_rows<T, F>(&self, height: u32, map_row: F) -> Vec<T>
    where
        T: Send,
        F: Fn(u32) -> T + Sync,
    {
        (0..height).map(map_row).collect()
    }
}

/// Compares the shaded [`BoxView`](crate::BoxView) of `scene` through two of
/// its views, the left eye's first, with the reference's image of each under
/// `bloom` over a `width` by `height` image. Returns a line an eye, or one
/// line when both eyes take one view. A line counts the pixels the
/// reference hits something opaque in, how many of them have a channel off
/// by more than `TOLERANCE`, and the mean difference of a channel. The
/// view takes `page_bytes` as [`view_image`] does. Errors unless the views
/// are the scene's and the device has multiview.
pub fn view_check(
    gpu: &Gpu,
    scene: &RenderScene,
    bloom: RenderBloom,
    view_ids: [U32Id<BRenderView>; 2],
    width: u32,
    height: u32,
    page_bytes: Option<u64>,
) -> Result<String> {
    let tiles = TileScene::of(scene, BoxView::draws_glass(&gpu.device))?;

    let texels = view_image(
        gpu,
        &tiles,
        scene,
        bloom,
        view_ids,
        [width, height],
        page_bytes,
    )?;
    let black = TyLinSrgbF32::new(0.0, 0.0, 0.0);

    let eyes: &[(&str, usize)] = if view_ids[0] == view_ids[1] {
        &[("", 0)]
    } else {
        &[("left eye: ", 0), ("right eye: ", 1)]
    };

    let mut report = String::new();

    for &(label, eye) in eyes {
        let image = render(
            &SerialRows,
            scene,
            view_ids[eye],
            RenderOcclusion::Corner,
            bloom,
            width,
            height,
        )
        .map_err(Error::Render)?;

        let reference = RenderOutput::from_image(&image, Some(TySrgbU8::new(0, 0, 0)));
        let (mut pixels, mut off, mut sum) = (0u64, 0u64, 0u64);

        for ((pixel, want), got) in image
            .pixels()
            .iter()
            .zip(reference.pixels())
            .zip(texels[eye].chunks_exact(4))
        {
            if pixel.transmittance != black {
                continue;
            }

            let differences = [want.red, want.green, want.blue]
                .into_iter()
                .zip(got)
                .map(|(want, &got)| want.abs_diff(got));

            pixels += 1;
            off += u64::from(differences.clone().any(|difference| difference > TOLERANCE));
            sum += differences.map(u64::from).sum::<u64>();
        }

        report += &format!(
            "{label}{pixels} opaque pixels, {off} off by over {TOLERANCE} ({:.3}%), mean difference {:.3} of 255\n",
            off as f64 / pixels.max(1) as f64 * 100.0,
            sum as f64 / (pixels.max(1) * 3) as f64
        );
    }

    Ok(report)
}
