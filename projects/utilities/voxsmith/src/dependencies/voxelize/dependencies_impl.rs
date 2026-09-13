use crate::{
    dependencies::{DependenciesImpl, voxelize::DecodeImage},
    operations::voxelize::DecodedImage,
};
use meshdoc::MeshImageMediaType;
use png::{ColorType, Decoder, Transformations};
use std::io::Cursor;
use zune_jpeg::{
    JpegDecoder,
    zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions},
};

impl DecodeImage for DependenciesImpl {
    fn decode_image(
        &self,
        media_type: MeshImageMediaType,
        bytes: &[u8],
    ) -> Result<DecodedImage, String> {
        match media_type {
            MeshImageMediaType::Png => decode_png(bytes),
            MeshImageMediaType::Jpeg => decode_jpeg(bytes),
        }
    }
}

/// A PNG of any color type and depth as 8-bit RGBA.
fn decode_png(bytes: &[u8]) -> Result<DecodedImage, String> {
    let mut decoder = Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|error| error.to_string())?;

    let size = reader
        .output_buffer_size()
        .ok_or_else(|| "png dimensions too large".to_owned())?;
    let mut buffer = vec![0u8; size];
    let frame = reader
        .next_frame(&mut buffer)
        .map_err(|error| error.to_string())?;
    let samples = &buffer[..frame.buffer_size()];

    let pixels = match reader.output_color_type().0 {
        ColorType::Rgba => samples
            .chunks_exact(4)
            .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
            .collect(),
        ColorType::Rgb => samples
            .chunks_exact(3)
            .map(|pixel| [pixel[0], pixel[1], pixel[2], 255])
            .collect(),
        ColorType::GrayscaleAlpha => samples
            .chunks_exact(2)
            .map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
            .collect(),
        ColorType::Grayscale => samples
            .iter()
            .map(|&gray| [gray, gray, gray, 255])
            .collect(),
        ColorType::Indexed => {
            return Err("png stayed indexed after normalization".to_owned());
        }
    };

    Ok(DecodedImage {
        width: frame.width,
        height: frame.height,
        pixels,
    })
}

/// A JPEG as 8-bit RGBA, opaque.
fn decode_jpeg(bytes: &[u8]) -> Result<DecodedImage, String> {
    let options = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(bytes), options);

    let samples = decoder.decode().map_err(|error| error.to_string())?;

    let info = decoder
        .info()
        .ok_or_else(|| "jpeg has no frame header".to_owned())?;

    Ok(DecodedImage {
        width: u32::from(info.width),
        height: u32::from(info.height),
        pixels: samples
            .chunks_exact(4)
            .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        dependencies::{DependenciesImpl, voxelize::DecodeImage},
        operations::voxelize::DecodedImage,
    };
    use meshdoc::MeshImageMediaType;
    use png::{BitDepth, ColorType, Encoder};

    /// A PNG of `color_type` at 8 bits holding `samples`.
    fn png(width: u32, height: u32, color_type: ColorType, samples: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut encoder = Encoder::new(&mut bytes, width, height);
        encoder.set_color(color_type);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(samples).unwrap();
        writer.finish().unwrap();
        bytes
    }

    #[test]
    fn png_decodes_each_color_type_to_rgba() {
        let cases = [
            (ColorType::Rgba, vec![1, 2, 3, 4], [1, 2, 3, 4]),
            (ColorType::Rgb, vec![1, 2, 3], [1, 2, 3, 255]),
            (ColorType::GrayscaleAlpha, vec![7, 9], [7, 7, 7, 9]),
            (ColorType::Grayscale, vec![7], [7, 7, 7, 255]),
        ];

        for (color_type, samples, pixel) in cases {
            assert_eq!(
                DependenciesImpl
                    .decode_image(MeshImageMediaType::Png, &png(1, 1, color_type, &samples))
                    .unwrap(),
                DecodedImage {
                    width: 1,
                    height: 1,
                    pixels: vec![pixel],
                },
                "{color_type:?}"
            );
        }
    }

    #[test]
    fn bytes_that_are_not_an_image_error() {
        for media_type in [MeshImageMediaType::Png, MeshImageMediaType::Jpeg] {
            assert!(
                DependenciesImpl
                    .decode_image(media_type, b"not an image")
                    .is_err()
            );
        }
    }
}
