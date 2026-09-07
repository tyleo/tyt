use crate::{
    dependencies::{
        DependenciesImpl,
        mesh::{EncodeBase64, EncodePng},
    },
    operations::mesh::AtlasImage,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use png::{BitDepth, ColorType, Encoder};

impl EncodeBase64 for DependenciesImpl {
    fn encode_base64(&self, bytes: &[u8]) -> String {
        STANDARD.encode(bytes)
    }
}

impl EncodePng for DependenciesImpl {
    fn encode_png(&self, image: &AtlasImage) -> Result<Vec<u8>, String> {
        let samples: Vec<u8> = image.pixels.iter().flatten().copied().collect();
        let mut out = Vec::new();
        let mut encoder = Encoder::new(&mut out, image.width, image.height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
        writer
            .write_image_data(&samples)
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
            mesh::{EncodeBase64, EncodePng},
        },
        operations::mesh::AtlasImage,
    };
    use png::Decoder;
    use std::io::Cursor;

    #[test]
    fn base64_pads() {
        assert_eq!(DependenciesImpl.encode_base64(&[0xC0]), "wA==");
    }

    #[test]
    fn png_round_trips_pixels() {
        let image = AtlasImage {
            width: 2,
            height: 2,
            pixels: vec![[1, 2, 3, 255], [4, 5, 6, 0], [7, 8, 9, 128], [0, 0, 0, 1]],
        };

        let bytes = DependenciesImpl.encode_png(&image).unwrap();

        let mut reader = Decoder::new(Cursor::new(bytes)).read_info().unwrap();
        let mut buffer = vec![0u8; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buffer).unwrap();
        assert_eq!((info.width, info.height), (2, 2));
        let samples: Vec<u8> = image.pixels.iter().flatten().copied().collect();
        assert_eq!(&buffer[..info.buffer_size()], &samples[..]);
    }

    #[test]
    fn png_rejects_a_zero_sized_image() {
        assert!(DependenciesImpl.encode_png(&AtlasImage::default()).is_err());
    }
}
