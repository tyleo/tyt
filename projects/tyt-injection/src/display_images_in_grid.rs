use crate::{base_config, with_capability_probe_guard};
use crossterm::terminal::size;
use image::{DynamicImage, RgbaImage, imageops};
use std::{
    io::{Error as IOError, ErrorKind, Result},
    path::Path,
};
use viuer::{Config, print};

/// Composes the images into a `columns`-wide grid — filled left to right, top to
/// bottom, with a transparent gutter between cells — and prints it inline as a
/// single image, scaled so its width spans `side_percent` percent of the
/// terminal's shorter visual side. This keeps a full set of thumbnails on one
/// screen rather than stacking them down the scrollback. Renders nothing when
/// `paths` is empty.
pub fn display_images_in_grid(paths: &[&Path], columns: u32, side_percent: u32) -> Result<()> {
    let columns = columns.max(1);
    let images = paths
        .iter()
        .map(|path| {
            image::open(path)
                .map(DynamicImage::into_rgba8)
                .map_err(|error| IOError::new(ErrorKind::InvalidData, error))
        })
        .collect::<Result<Vec<_>>>()?;
    let Some(grid) = compose_grid(&images, columns) else {
        return Ok(());
    };

    // viuer measures width in terminal columns and preserves aspect ratio. The
    // terminal's shorter visual side is whichever is smaller: its width (one
    // cell-width per column) or its height (a cell is roughly twice as tall as
    // wide, so two cell-widths per row). Size the grid to a fraction of it, and
    // never wider than the terminal itself.
    let (term_cols, term_rows) = size().unwrap_or((80, 24));
    let shortest = (term_cols as u32).min((term_rows as u32) * 2).max(1);
    let width = (shortest * side_percent / 100).clamp(1, term_cols as u32);
    let cfg = Config {
        width: Some(width),
        ..base_config()
    };
    with_capability_probe_guard(|| print(&grid, &cfg).map(|_| ()).map_err(IOError::other))
}

/// Lays the images out in a `columns`-wide grid of uniform cells — each sized to
/// the largest source image and separated by a transparent gutter — returning
/// the composited image, or `None` when there are no images.
fn compose_grid(images: &[RgbaImage], columns: u32) -> Option<DynamicImage> {
    let cell_w = images.iter().map(RgbaImage::width).max()?;
    let cell_h = images.iter().map(RgbaImage::height).max()?;
    // A gutter scaled to the cell so the spacing survives the final downscale.
    let gap = (cell_w.min(cell_h) / 16).max(1);
    let rows = (images.len() as u32).div_ceil(columns);

    let total_w = columns * cell_w + (columns - 1) * gap;
    let total_h = rows * cell_h + (rows - 1) * gap;
    let mut canvas = RgbaImage::new(total_w, total_h);

    for (index, image) in images.iter().enumerate() {
        let index = index as u32;
        let cell_x = (index % columns) * (cell_w + gap);
        let cell_y = (index / columns) * (cell_h + gap);
        // Center each image in its cell so mixed sizes stay aligned.
        let x = cell_x + (cell_w - image.width()) / 2;
        let y = cell_y + (cell_h - image.height()) / 2;
        imageops::overlay(&mut canvas, image, x as i64, y as i64);
    }

    Some(DynamicImage::ImageRgba8(canvas))
}
