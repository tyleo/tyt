use crate::{
    dependencies::{
        DependenciesImpl,
        mesh::{EncodePng, PngChannels, PngImage},
    },
    operations::mesh::Transfer,
};
use png::{BitDepth, ColorType, Encoder, ScaledFloat, SourceChromaticities, SrgbRenderingIntent};

impl EncodePng for DependenciesImpl {
    fn encode_png(&self, image: &PngImage) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        let mut encoder = Encoder::new(&mut out, image.width, image.height);

        encoder.set_color(match image.channels {
            PngChannels::Grey => ColorType::Grayscale,
            PngChannels::GreyAlpha => ColorType::GrayscaleAlpha,
            PngChannels::Rgb => ColorType::Rgb,
            PngChannels::Rgba => ColorType::Rgba,
        });
        encoder.set_depth(BitDepth::Eight);

        match image.transfer {
            Transfer::Linear => encoder.set_source_gamma(ScaledFloat::from_scaled(100_000)),

            Transfer::Srgb => {
                // The `sRGB` chunk's recommended `gAMA` and `cHRM` fallbacks.
                // The encoder writes them only at these values.
                encoder.set_source_gamma(ScaledFloat::from_scaled(45_455));
                encoder.set_source_chromaticities(SourceChromaticities {
                    white: (
                        ScaledFloat::from_scaled(31_270),
                        ScaledFloat::from_scaled(32_900),
                    ),
                    red: (
                        ScaledFloat::from_scaled(64_000),
                        ScaledFloat::from_scaled(33_000),
                    ),
                    green: (
                        ScaledFloat::from_scaled(30_000),
                        ScaledFloat::from_scaled(60_000),
                    ),
                    blue: (
                        ScaledFloat::from_scaled(15_000),
                        ScaledFloat::from_scaled(6_000),
                    ),
                });
                encoder.set_source_srgb(SrgbRenderingIntent::Perceptual);
            }
        }

        let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
        writer
            .write_image_data(&image.samples)
            .map_err(|error| error.to_string())?;
        writer.finish().map_err(|error| error.to_string())?;

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dependencies::{
            DependenciesImpl,
            mesh::{EncodePng, PngChannels, PngImage},
        },
        operations::mesh::Transfer,
    };
    use png::{ColorType, Decoder, Info, ScaledFloat, SrgbRenderingIntent};
    use std::io::Cursor;

    /// A 2 x 1 image of `channels` under `transfer`, its samples counting up.
    fn image(channels: PngChannels, transfer: Transfer) -> PngImage {
        PngImage {
            width: 2,
            height: 1,
            channels,
            transfer,
            samples: (1..=2 * channels.count() as u8).collect(),
        }
    }

    /// The decoded samples and header of `bytes`.
    fn decode(bytes: Vec<u8>) -> (Vec<u8>, Info<'static>) {
        let mut reader = Decoder::new(Cursor::new(bytes)).read_info().unwrap();
        let mut buffer = vec![0u8; reader.output_buffer_size().unwrap()];
        let output = reader.next_frame(&mut buffer).unwrap();
        buffer.truncate(output.buffer_size());
        (buffer, reader.info().clone())
    }

    #[test]
    fn each_channel_format_round_trips_its_samples() {
        for (channels, color_type) in [
            (PngChannels::Grey, ColorType::Grayscale),
            (PngChannels::GreyAlpha, ColorType::GrayscaleAlpha),
            (PngChannels::Rgb, ColorType::Rgb),
            (PngChannels::Rgba, ColorType::Rgba),
        ] {
            let image = image(channels, Transfer::Linear);

            let (samples, info) = decode(DependenciesImpl.encode_png(&image).unwrap());

            assert_eq!(samples, image.samples, "{channels:?}");
            assert_eq!(info.color_type, color_type, "{channels:?}");
            assert_eq!((info.width, info.height), (2, 1), "{channels:?}");
        }
    }

    #[test]
    fn linear_stamps_a_gamma_of_one_and_no_srgb_chunk() {
        let bytes = DependenciesImpl
            .encode_png(&image(PngChannels::Grey, Transfer::Linear))
            .unwrap();

        let (_, info) = decode(bytes);

        assert_eq!(info.gama_chunk, Some(ScaledFloat::from_scaled(100_000)));
        assert_eq!(info.srgb, None);
        assert_eq!(info.chrm_chunk, None);
    }

    #[test]
    fn srgb_stamps_the_srgb_chunk_with_its_fallbacks() {
        let bytes = DependenciesImpl
            .encode_png(&image(PngChannels::Rgb, Transfer::Srgb))
            .unwrap();

        let (_, info) = decode(bytes);

        assert_eq!(info.srgb, Some(SrgbRenderingIntent::Perceptual));
        assert_eq!(info.gama_chunk, Some(ScaledFloat::from_scaled(45_455)));
        let chromaticities = info.chrm_chunk.unwrap();
        assert_eq!(
            chromaticities.white,
            (
                ScaledFloat::from_scaled(31_270),
                ScaledFloat::from_scaled(32_900)
            )
        );
    }

    #[test]
    fn a_sample_count_off_the_format_errors() {
        let mut image = image(PngChannels::Rgb, Transfer::Linear);
        image.samples.pop();

        assert!(DependenciesImpl.encode_png(&image).is_err());
    }

    #[test]
    fn a_zero_sized_image_errors() {
        let image = PngImage {
            width: 0,
            height: 0,
            channels: PngChannels::Rgba,
            transfer: Transfer::Linear,
            samples: Vec::new(),
        };

        assert!(DependenciesImpl.encode_png(&image).is_err());
    }
}
