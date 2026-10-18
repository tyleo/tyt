use crate::{Error, RenderImage, Result, tonemap};
use ty_math::{TyLinSrgbF32, TySrgbF32, TySrgbU8, TySrgbaF32, TySrgbaU8};

/// An 8-bit sRGB image with straight alpha that a PNG can store. Rows run top
/// to bottom, and pixels run left to right within a row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderOutput {
    width: u32,

    height: u32,

    pixels: Vec<TySrgbaU8>,
}

impl RenderOutput {
    /// Encodes `image` through [`tonemap`] and the sRGB transfer. A pixel's
    /// coverage is the larger of one minus its peak transmittance and the
    /// peak of its light, clamped to one. Its layer is the light over that
    /// coverage, tonemapped, times the coverage. Under `background` the pixel
    /// is the layer plus the background scaled by the transmittance, clamped
    /// to one and written at full alpha. Without one the pixel is solved to be
    /// exact over white for a viewer that blends sRGB values.
    pub fn from_image(image: &RenderImage, background: Option<TySrgbU8>) -> Self {
        let backdrop = background.map(|color| color.into_format::<f32>().into_linear());

        let pixels = image
            .pixels()
            .iter()
            .map(|pixel| {
                let coverage = (1.0 - peak(pixel.transmittance))
                    .max(peak(pixel.light))
                    .min(1.0);

                // The coverage is at least the light's peak, so a zero
                // coverage carries no light.
                let color = if coverage > 0.0 {
                    pixel.light / coverage
                } else {
                    pixel.light
                };

                let layer = tonemap(color) * coverage;

                match backdrop {
                    Some(backdrop) => {
                        let over =
                            TySrgbF32::from_linear(clamped(layer + backdrop * pixel.transmittance));

                        TySrgbaF32::new(over.red, over.green, over.blue, 1.0)
                            .into_format::<u8, u8>()
                    }

                    None => over_white(layer, pixel.transmittance),
                }
            })
            .collect();

        RenderOutput {
            width: image.width(),
            height: image.height(),
            pixels,
        }
    }

    /// Tiles `tiles` row by row into a near-square sheet with a border
    /// between them. A slot past the last tile takes `background`, or stays
    /// transparent without one.
    pub fn tiled(
        tiles: &[&RenderOutput],
        border_width: u32,
        border_color: TySrgbU8,
        background: Option<TySrgbU8>,
    ) -> Result<Self> {
        let first = tiles.first().ok_or(Error::EmptySheet)?;
        let (width, height) = (first.width, first.height);

        if let Some(tile) = tiles
            .iter()
            .find(|tile| (tile.width, tile.height) != (width, height))
        {
            return Err(Error::SheetTileSize {
                width: tile.width,
                height: tile.height,
                first_width: width,
                first_height: height,
            });
        }

        let count = tiles.len() as u32;
        let columns = count.isqrt() + u32::from(count.isqrt().pow(2) < count);
        let rows = count.div_ceil(columns);
        let sheet_width = columns * width + (columns - 1) * border_width;
        let sheet_height = rows * height + (rows - 1) * border_width;

        let opaque = |color: TySrgbU8| TySrgbaU8::new(color.red, color.green, color.blue, 255);
        let empty = background.map_or(TySrgbaU8::new(0, 0, 0, 0), opaque);

        let mut pixels = vec![opaque(border_color); (sheet_width * sheet_height) as usize];

        for slot in 0..columns * rows {
            let left = slot % columns * (width + border_width);
            let top = slot / columns * (height + border_width);

            for y in 0..height {
                let start = ((top + y) * sheet_width + left) as usize;
                let row = &mut pixels[start..start + width as usize];

                match tiles.get(slot as usize) {
                    Some(tile) => row.copy_from_slice(
                        &tile.pixels[(y * width) as usize..((y + 1) * width) as usize],
                    ),

                    None => row.fill(empty),
                }
            }
        }

        Ok(RenderOutput {
            width: sheet_width,
            height: sheet_height,
            pixels,
        })
    }

    /// The width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Every pixel in row-major order.
    pub fn pixels(&self) -> &[TySrgbaU8] {
        &self.pixels
    }

    /// Every pixel's four bytes in row-major order, red first.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.pixels
            .iter()
            .flat_map(|pixel| <[u8; 4]>::from(*pixel))
            .collect()
    }
}

fn peak(color: TyLinSrgbF32) -> f32 {
    color.red.max(color.green).max(color.blue)
}

fn clamped(color: TyLinSrgbF32) -> TyLinSrgbF32 {
    TyLinSrgbF32::new(
        color.red.min(1.0),
        color.green.min(1.0),
        color.blue.min(1.0),
    )
}

/// Solves `layer` over `transmittance` so a viewer blending sRGB values lays
/// it over white exactly.
fn over_white(layer: TyLinSrgbF32, transmittance: TyLinSrgbF32) -> TySrgbaU8 {
    let white = TySrgbF32::from_linear(clamped(layer + transmittance));

    let least = white.red.min(white.green).min(white.blue);
    let clear = peak(transmittance).min(least);
    let alpha = 1.0 - clear;

    // Each color falls between zero and one because every channel over white
    // lies between the transparency and one.
    let color = |channel: f32| {
        if alpha > 0.0 {
            (channel - clear) / alpha
        } else {
            0.0
        }
    };

    TySrgbaF32::new(
        color(white.red),
        color(white.green),
        color(white.blue),
        alpha,
    )
    .into_format::<u8, u8>()
}

#[cfg(test)]
mod tests {
    use crate::{Error, RenderImage, RenderOutput, RenderPixel, tonemap};
    use ty_math::{TyLinSrgbF32, TyLinSrgbaF32, TySrgbU8, TySrgbaF32, TySrgbaU8};

    const BLACK: TyLinSrgbF32 = TyLinSrgbF32::new(0.0, 0.0, 0.0);

    fn encoded(color: TyLinSrgbF32, alpha: f32) -> TySrgbaU8 {
        TySrgbaF32::from_linear(TyLinSrgbaF32::new(
            color.red,
            color.green,
            color.blue,
            alpha,
        ))
        .into_format::<u8, u8>()
    }

    #[test]
    fn hits_encode_through_the_tonemap_and_misses_take_the_background() {
        let mut image = RenderImage::new(2, 1);
        image.set_pixel(
            0,
            0,
            RenderPixel {
                light: TyLinSrgbF32::new(0.5, 0.25, 0.0),
                transmittance: BLACK,
            },
        );

        let expected = encoded(tonemap(TyLinSrgbF32::new(0.5, 0.25, 0.0)), 1.0);

        let transparent = RenderOutput::from_image(&image, None);
        assert_eq!(transparent.width(), 2);
        assert_eq!(transparent.height(), 1);
        assert_eq!(transparent.pixels(), [expected, TySrgbaU8::new(0, 0, 0, 0)]);
        assert_eq!(transparent.to_bytes().len(), 8);
        assert_eq!(&transparent.to_bytes()[4..], [0, 0, 0, 0]);

        let backed = RenderOutput::from_image(&image, Some(TySrgbU8::new(10, 20, 30)));
        assert_eq!(backed.pixels()[1], TySrgbaU8::new(10, 20, 30, 255));
    }

    #[test]
    fn a_halo_pixel_composites_over_the_background() {
        // A halo of (0.5, 0.25, 0) over a miss covers it by its peak.
        let mut image = RenderImage::new(1, 1);
        image.set_pixel(
            0,
            0,
            RenderPixel {
                light: TyLinSrgbF32::new(0.5, 0.25, 0.0),
                transmittance: TyLinSrgbF32::new(0.5, 0.5, 0.5),
            },
        );

        let mapped = tonemap(TyLinSrgbF32::new(1.0, 0.5, 0.0));

        // Over black the color halves to the halo's light.
        let over_black = RenderOutput::from_image(&image, Some(TySrgbU8::new(0, 0, 0)));
        assert_eq!(over_black.pixels(), [encoded(mapped / 2.0, 1.0)]);

        // Over white the halo lifts the color toward white.
        let over_white = RenderOutput::from_image(&image, Some(TySrgbU8::new(255, 255, 255)));
        let [red, green, blue, alpha] = <[u8; 4]>::from(over_white.pixels()[0]);
        assert_eq!(alpha, 255);
        assert!(
            red > green && green > blue && blue > 128,
            "{red} {green} {blue}"
        );
    }

    #[test]
    fn the_transparent_output_laid_over_white_matches_a_white_background() {
        let reflection = TyLinSrgbF32::new(0.04, 0.04, 0.04);
        let clear = TyLinSrgbF32::new(0.96, 0.96, 0.96);
        let pane = TyLinSrgbF32::new(0.75, 0.75, 0.75);

        let pixels = [
            // Red glass.
            (reflection, TyLinSrgbF32::new(0.96, 0.0, 0.0)),
            (reflection, clear),
            (TyLinSrgbF32::new(0.125, 0.125, 0.125), pane),
            // A highlight brighter than the pane's coverage.
            (TyLinSrgbF32::new(0.5, 0.25, 0.25), pane),
            // A halo over a miss.
            (
                TyLinSrgbF32::new(0.5, 0.25, 0.0),
                TyLinSrgbF32::new(0.5, 0.5, 0.5),
            ),
            // An opaque hit.
            (TyLinSrgbF32::new(0.5, 0.25, 0.0), BLACK),
            // A miss.
            (BLACK, TyLinSrgbF32::new(1.0, 1.0, 1.0)),
        ];

        let mut image = RenderImage::new(pixels.len() as u32, 1);

        for (x, (light, transmittance)) in pixels.into_iter().enumerate() {
            image.set_pixel(
                x as u32,
                0,
                RenderPixel {
                    light,
                    transmittance,
                },
            );
        }

        let transparent = RenderOutput::from_image(&image, None);
        let white = RenderOutput::from_image(&image, Some(TySrgbU8::new(255, 255, 255)));

        for (pixel, expected) in transparent.pixels().iter().zip(white.pixels()) {
            let [red, green, blue, alpha] = <[u8; 4]>::from(*pixel);
            let [expected_red, expected_green, expected_blue, _] = <[u8; 4]>::from(*expected);

            // A viewer blending sRGB values lays the pixel over white.
            let alpha = f32::from(alpha) / 255.0;
            let laid = |channel: u8| f32::from(channel) * alpha + 255.0 * (1.0 - alpha);

            for (channel, expected) in [
                (red, expected_red),
                (green, expected_green),
                (blue, expected_blue),
            ] {
                assert!(
                    (laid(channel) - f32::from(expected)).abs() <= 1.0,
                    "{pixel:?} lays over white at {} where the background gives {expected:?}",
                    laid(channel)
                );
            }
        }
    }

    #[test]
    fn colorless_transmittance_sets_the_alpha() {
        let pane = TyLinSrgbF32::new(0.75, 0.75, 0.75);

        let mut image = RenderImage::new(2, 1);
        image.set_pixel(
            0,
            0,
            RenderPixel {
                light: TyLinSrgbF32::new(0.125, 0.125, 0.125),
                transmittance: pane,
            },
        );
        // A highlight brighter than the pane's coverage.
        image.set_pixel(
            1,
            0,
            RenderPixel {
                light: TyLinSrgbF32::new(0.5, 0.25, 0.25),
                transmittance: pane,
            },
        );

        let output = RenderOutput::from_image(&image, None);

        let [dim, bright] = [output.pixels()[0], output.pixels()[1]];
        assert_eq!((dim.alpha, bright.alpha), (64, 64));
        assert!(bright.red > dim.red, "{dim:?} {bright:?}");
    }

    #[test]
    fn red_glass_over_white_is_red_and_clear_glass_over_white_stays_white() {
        let reflection = TyLinSrgbF32::new(0.04, 0.04, 0.04);
        let clear = TyLinSrgbF32::new(0.96, 0.96, 0.96);

        let mut image = RenderImage::new(3, 1);
        image.set_pixel(
            0,
            0,
            RenderPixel {
                light: reflection,
                transmittance: TyLinSrgbF32::new(0.96, 0.0, 0.0),
            },
        );
        image.set_pixel(
            1,
            0,
            RenderPixel {
                light: reflection,
                transmittance: clear,
            },
        );
        // A highlight on clear glass over white clamps to white.
        image.set_pixel(
            2,
            0,
            RenderPixel {
                light: TyLinSrgbF32::new(0.5, 0.5, 0.5),
                transmittance: clear,
            },
        );

        let output = RenderOutput::from_image(&image, Some(TySrgbU8::new(255, 255, 255)));

        let [red, green, blue, alpha] = <[u8; 4]>::from(output.pixels()[0]);
        assert_eq!(alpha, 255);
        assert!(
            red > 250 && green < 64 && green == blue,
            "{red} {green} {blue}"
        );

        let [red, green, blue, alpha] = <[u8; 4]>::from(output.pixels()[1]);
        assert_eq!(alpha, 255);
        assert!(
            red > 250 && red == green && green == blue,
            "{red} {green} {blue}"
        );

        assert_eq!(output.pixels()[2], TySrgbaU8::new(255, 255, 255, 255));
    }

    /// A `size` by `size` tile filled with `color`.
    fn tile(size: u32, color: TySrgbU8) -> RenderOutput {
        RenderOutput::from_image(&RenderImage::new(size, size), Some(color))
    }

    #[test]
    fn five_tiles_fill_three_columns_and_two_rows_between_borders() {
        let colors = [1, 2, 3, 4, 5].map(|value| TySrgbU8::new(value, 0, 0));
        let tiles = colors.map(|color| tile(2, color));

        let sheet = RenderOutput::tiled(
            &tiles.each_ref(),
            1,
            TySrgbU8::new(128, 128, 128),
            Some(TySrgbU8::new(9, 9, 9)),
        )
        .unwrap();

        assert_eq!((sheet.width(), sheet.height()), (8, 5));

        let pixel = |x: u32, y: u32| <[u8; 4]>::from(sheet.pixels()[(y * 8 + x) as usize]);
        assert_eq!(pixel(0, 0), [1, 0, 0, 255]);
        assert_eq!(pixel(2, 0), [128, 128, 128, 255]);
        assert_eq!(pixel(3, 1), [2, 0, 0, 255]);
        assert_eq!(pixel(6, 1), [3, 0, 0, 255]);
        assert_eq!(pixel(0, 2), [128, 128, 128, 255]);
        assert_eq!(pixel(1, 3), [4, 0, 0, 255]);
        assert_eq!(pixel(4, 4), [5, 0, 0, 255]);
        assert_eq!(pixel(7, 4), [9, 9, 9, 255]);
    }

    #[test]
    fn a_square_count_tiles_a_square_and_a_zero_border_abuts_the_tiles() {
        let black = TySrgbU8::new(0, 0, 0);
        let tiles = [0, 1, 2, 3].map(|_| tile(4, black));

        let bordered = RenderOutput::tiled(&tiles.each_ref(), 3, black, None).unwrap();
        assert_eq!((bordered.width(), bordered.height()), (11, 11));

        let abutting = RenderOutput::tiled(&tiles.each_ref(), 0, black, None).unwrap();
        assert_eq!((abutting.width(), abutting.height()), (8, 8));
    }

    #[test]
    fn a_sheet_needs_tiles_of_one_size() {
        let black = TySrgbU8::new(0, 0, 0);

        assert_eq!(
            RenderOutput::tiled(&[], 1, black, None),
            Err(Error::EmptySheet)
        );
        assert_eq!(
            RenderOutput::tiled(&[&tile(2, black), &tile(3, black)], 1, black, None),
            Err(Error::SheetTileSize {
                width: 3,
                height: 3,
                first_width: 2,
                first_height: 2,
            })
        );
    }
}
