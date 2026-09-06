use crate::{DecodePng, EncodePng, GoxlRgbaImage};
use png::{BitDepth, ColorType, Decoder, Encoder};
use std::io::Cursor;

/// The dependencies over `png`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl DecodePng for DependenciesImpl {
    /// Rejects a PNG that is not 8-bit RGBA, the only form Goxel writes.
    fn decode_png(&self, bytes: &[u8]) -> Result<GoxlRgbaImage, String> {
        let decoder = Decoder::new(Cursor::new(bytes));
        let mut reader = decoder.read_info().map_err(|error| error.to_string())?;

        let info = reader.info();
        let (color_type, bit_depth) = (info.color_type, info.bit_depth);
        if color_type != ColorType::Rgba || bit_depth != BitDepth::Eight {
            return Err(format!(
                "expected an 8-bit RGBA PNG, found {color_type:?} at {bit_depth:?}"
            ));
        }

        let size = reader
            .output_buffer_size()
            .ok_or_else(|| "png dimensions too large".to_owned())?;
        let mut buffer = vec![0u8; size];
        let frame = reader
            .next_frame(&mut buffer)
            .map_err(|error| error.to_string())?;
        let pixels = buffer[..frame.buffer_size()]
            .chunks_exact(4)
            .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
            .collect();

        Ok(GoxlRgbaImage {
            width: frame.width,
            height: frame.height,
            pixels,
        })
    }
}

impl EncodePng for DependenciesImpl {
    fn encode_png(&self, image: &GoxlRgbaImage) -> Vec<u8> {
        let samples: Vec<u8> = image.pixels.iter().flatten().copied().collect();
        let mut out = Vec::new();
        let mut encoder = Encoder::new(&mut out, image.width, image.height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);

        // The writer guarantees a non-zero-sized image holding `width * height`
        // pixels, the only inputs an in-memory encode rejects.
        let mut writer = encoder
            .write_header()
            .expect("a non-zero-sized RGBA header writes to an in-memory buffer");
        writer
            .write_image_data(&samples)
            .expect("width * height pixels write to an in-memory buffer");
        writer
            .finish()
            .expect("finishing an in-memory PNG is infallible");

        out
    }
}

#[cfg(test)]
mod tests {
    use crate::{DecodePng, DependenciesImpl, EncodePng, GoxlRgbaImage};
    use png::{BitDepth, ColorType, Encoder};

    #[test]
    fn png_round_trips_pixels() {
        let image = GoxlRgbaImage {
            width: 2,
            height: 2,
            pixels: vec![[1, 2, 3, 255], [4, 5, 6, 0], [7, 8, 9, 128], [0, 0, 0, 1]],
        };

        let bytes = DependenciesImpl.encode_png(&image);
        assert_eq!(DependenciesImpl.decode_png(&bytes).unwrap(), image);
    }

    #[test]
    fn png_rejects_a_non_rgba_image() {
        // A 1x1 8-bit RGB PNG: valid, but not the RGBA form Goxel writes.
        let mut bytes = Vec::new();
        let mut encoder = Encoder::new(&mut bytes, 1, 1);
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&[1, 2, 3]).unwrap();
        writer.finish().unwrap();

        assert!(DependenciesImpl.decode_png(&bytes).is_err());
    }

    #[test]
    fn png_rejects_bytes_that_are_not_a_png() {
        assert!(DependenciesImpl.decode_png(b"not a png").is_err());
    }
}
