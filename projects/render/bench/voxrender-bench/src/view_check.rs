use crate::{Error, Gpu, Result, view_image};
use ty_math::{TyLinSrgbF32, TySrgbU8};
use voxrender::{RenderBloom, RenderOcclusion, RenderOutput, RenderScene};
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

/// Compares the shaded [`BoxView`](crate::BoxView) of `scene`'s first view
/// with the reference's image of it, over a `size` square image, and
/// returns a line of the result. The line counts the pixels the reference
/// hits something opaque in, how many of them have a channel off by over
/// 8 of 255 steps, and the mean difference of a channel. Errors unless
/// the scene has a view and the device has multiview.
pub fn view_check(gpu: &Gpu, scene: &RenderScene, size: u32) -> Result<String> {
    let (view_id, _) = scene.iter_views().next().ok_or(Error::NoView)?;

    let image = render(
        &SerialRows,
        scene,
        view_id,
        RenderOcclusion::Corner,
        RenderBloom::default(),
        size,
        size,
    )
    .map_err(Error::Render)?;

    let reference = RenderOutput::from_image(&image, Some(TySrgbU8::new(0, 0, 0)));
    let texels = view_image(gpu, scene, size, size)?;
    let black = TyLinSrgbF32::new(0.0, 0.0, 0.0);

    let (mut pixels, mut off, mut sum) = (0u64, 0u64, 0u64);

    for ((pixel, want), got) in image
        .pixels()
        .iter()
        .zip(reference.pixels())
        .zip(texels.chunks_exact(4))
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

    Ok(format!(
        "{pixels} opaque pixels, {off} off by over {TOLERANCE} ({:.3}%), mean difference {:.3} of 255\n",
        off as f64 / pixels.max(1) as f64 * 100.0,
        sum as f64 / (pixels.max(1) * 3) as f64
    ))
}
