use crate::{
    CompressLzfse, DecodePng, DecodeVMaxPlist, DecodeVMaxSceneJson, DecompressLzfse, EncodePng,
    EncodeVMaxPlist, EncodeVMaxSceneJson,
};
use lzfse::Error as LzfseError;
use png::{
    BitDepth, ColorType, Decoder, Encoder, Filter, SrgbRenderingIntent, Transformations, chunk,
};
use serde::{Serialize, de::DeserializeOwned};
use std::io::Cursor;
use vmax::{
    VMaxContentsVmaxbFile, VMaxHistoryVmaxhbFile, VMaxHistoryVmaxhvsbFile, VMaxHistoryVmaxhvscFile,
    VMaxImage, VMaxPalettePngFile, VMaxPaletteSettingsVmaxpsbFile, VMaxSceneJsonFile,
};

/// Static Exif block Voxel Max embeds in every `palette*.png`: a big-endian
/// TIFF whose Exif IFD records the sRGB color space plus the image dimensions.
/// Bytes `48..52` hold `PixelXDimension` and `60..64` hold `PixelYDimension`.
/// Only the width is patched per image because the height is always 1.
const EXIF: [u8; 68] = [
    0x4d, 0x4d, 0x00, 0x2a, 0x00, 0x00, 0x00, 0x08, 0x00, 0x01, 0x87, 0x69, 0x00, 0x04, 0x00, 0x00,
    0x00, 0x01, 0x00, 0x00, 0x00, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xa0, 0x01, 0x00, 0x03,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xa0, 0x02, 0x00, 0x04, 0x00, 0x00, 0x00, 0x01,
    0x00, 0x00, 0x01, 0x00, 0xa0, 0x03, 0x00, 0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    0x00, 0x00, 0x00, 0x00,
];

/// The dependencies over `lzfse`, `plist`, `png`, and `serde_json`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

fn decode_plist<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    plist::from_bytes(bytes).map_err(|error| error.to_string())
}

fn encode_plist<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    plist::to_writer_binary(&mut bytes, value).map_err(|error| error.to_string())?;
    Ok(bytes)
}

impl CompressLzfse for DependenciesImpl {
    fn compress_lzfse(&self, bytes: &[u8]) -> Vec<u8> {
        let mut capacity = bytes
            .len()
            .saturating_add(bytes.len() / 16)
            .saturating_add(4096);
        // The loop ends because LZFSE always succeeds given a large enough
        // buffer.
        loop {
            let mut out = vec![0u8; capacity];
            if let Ok(len) = lzfse::encode_buffer(bytes, &mut out) {
                out.truncate(len);
                return out;
            }
            capacity = capacity.saturating_mul(2);
        }
    }
}

impl DecompressLzfse for DependenciesImpl {
    fn decompress_lzfse(&self, stream: &[u8]) -> Result<Vec<u8>, String> {
        let mut capacity = stream.len().saturating_mul(8).max(4096);
        // The ceiling caps how much memory a stream can demand.
        let ceiling = stream.len().saturating_mul(8192).max(1 << 20);
        loop {
            let mut out = vec![0u8; capacity];
            match lzfse::decode_buffer(stream, &mut out) {
                Ok(len) => {
                    out.truncate(len);
                    return Ok(out);
                }
                // A full buffer may mean truncation, so grow and retry.
                Err(LzfseError::BufferTooSmall) if capacity < ceiling => {
                    capacity = capacity.saturating_mul(2);
                }
                Err(LzfseError::BufferTooSmall) => {
                    return Err(format!("lzfse output exceeds {ceiling} bytes"));
                }
                Err(LzfseError::CompressFailed) => return Err("malformed lzfse stream".to_owned()),
            }
        }
    }
}

impl DecodeVMaxPlist for DependenciesImpl {
    fn decode_contents_vmaxb(&self, bytes: &[u8]) -> Result<VMaxContentsVmaxbFile, String> {
        decode_plist(bytes)
    }

    fn decode_history_vmaxhb(&self, bytes: &[u8]) -> Result<VMaxHistoryVmaxhbFile, String> {
        decode_plist(bytes)
    }

    fn decode_history_vmaxhvsb(&self, bytes: &[u8]) -> Result<VMaxHistoryVmaxhvsbFile, String> {
        decode_plist(bytes)
    }

    fn decode_history_vmaxhvsc(&self, bytes: &[u8]) -> Result<VMaxHistoryVmaxhvscFile, String> {
        decode_plist(bytes)
    }

    fn decode_palette_settings_vmaxpsb(
        &self,
        bytes: &[u8],
    ) -> Result<VMaxPaletteSettingsVmaxpsbFile, String> {
        decode_plist(bytes)
    }
}

impl EncodeVMaxPlist for DependenciesImpl {
    fn encode_contents_vmaxb(&self, file: &VMaxContentsVmaxbFile) -> Result<Vec<u8>, String> {
        encode_plist(file)
    }

    fn encode_history_vmaxhb(&self, file: &VMaxHistoryVmaxhbFile) -> Result<Vec<u8>, String> {
        encode_plist(file)
    }

    fn encode_history_vmaxhvsb(&self, file: &VMaxHistoryVmaxhvsbFile) -> Result<Vec<u8>, String> {
        encode_plist(file)
    }

    fn encode_history_vmaxhvsc(&self, file: &VMaxHistoryVmaxhvscFile) -> Result<Vec<u8>, String> {
        encode_plist(file)
    }

    fn encode_palette_settings_vmaxpsb(
        &self,
        file: &VMaxPaletteSettingsVmaxpsbFile,
    ) -> Result<Vec<u8>, String> {
        encode_plist(file)
    }
}

impl DecodeVMaxSceneJson for DependenciesImpl {
    fn decode_vmax_scene_json(&self, bytes: &[u8]) -> Result<VMaxSceneJsonFile, String> {
        serde_json::from_slice(bytes).map_err(|error| error.to_string())
    }
}

impl EncodeVMaxSceneJson for DependenciesImpl {
    fn encode_vmax_scene_json(&self, file: &VMaxSceneJsonFile) -> Vec<u8> {
        serde_json::to_vec(file).expect("a scene holds nothing without a JSON form")
    }
}

impl DecodePng for DependenciesImpl {
    fn decode_png(&self, bytes: &[u8]) -> Result<VMaxImage, String> {
        let mut decoder = Decoder::new(Cursor::new(bytes));
        // Normalize paletted, sub-8-bit, and 16-bit inputs down to 8-bit
        // channels so each pixel reduces to a single `[r, g, b, a]` cell.
        decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
        let mut reader = decoder.read_info().map_err(|error| error.to_string())?;
        let size = reader
            .output_buffer_size()
            .ok_or_else(|| "png dimensions too large".to_owned())?;
        let mut buffer = vec![0; size];
        let info = reader
            .next_frame(&mut buffer)
            .map_err(|error| error.to_string())?;
        let pixels = buffer[..info.buffer_size()]
            .chunks_exact(info.color_type.samples())
            .map(|cell| match cell {
                [gray] => Ok([*gray, *gray, *gray, u8::MAX]),
                [gray, alpha] => Ok([*gray, *gray, *gray, *alpha]),
                [r, g, b] => Ok([*r, *g, *b, u8::MAX]),
                [r, g, b, alpha] => Ok([*r, *g, *b, *alpha]),
                _ => Err(format!(
                    "png pixels hold {} samples, not 1 to 4",
                    cell.len()
                )),
            })
            .collect::<Result<_, _>>()?;
        Ok(VMaxImage {
            width: info.width,
            height: info.height,
            pixels,
        })
    }
}

impl EncodePng for DependenciesImpl {
    fn encode_png(&self, image: &VMaxImage) -> Result<Vec<u8>, String> {
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

    fn encode_palette_png(&self, file: &VMaxPalettePngFile) -> Result<Vec<u8>, String> {
        let samples: Vec<u8> = file.0.iter().flatten().copied().collect();
        let mut out = Vec::new();
        let mut encoder = Encoder::new(&mut out, file.0.len() as u32, 1);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        // Match Voxel Max's encoder.
        encoder.set_filter(Filter::Sub);
        encoder.set_source_srgb(SrgbRenderingIntent::Perceptual);
        let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
        let mut exif = EXIF;
        exif[48..52].copy_from_slice(&(file.0.len() as u32).to_be_bytes());
        writer
            .write_chunk(chunk::eXIf, &exif)
            .map_err(|error| error.to_string())?;
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
        CompressLzfse, DecodePng, DecodeVMaxSceneJson, DecompressLzfse, DependenciesImpl,
        EncodePng, EncodeVMaxSceneJson, decompress_lzfse_or_raw,
    };
    use vmax::{VMaxImage, VMaxPalettePngFile, VMaxSceneJsonFile};

    #[test]
    fn lzfse_round_trips_and_frames_the_stream() {
        let bytes: Vec<u8> = (0..4096u32).map(|i| (i % 7) as u8).collect();
        let stream = DependenciesImpl.compress_lzfse(&bytes);
        assert!(stream.starts_with(b"bvx"));
        assert!(stream.len() < bytes.len());
        assert_eq!(DependenciesImpl.decompress_lzfse(&stream).unwrap(), bytes);
    }

    #[test]
    fn unframed_bytes_pass_through_untouched() {
        let raw = b"bplist00 not an lzfse stream";
        assert_eq!(decompress_lzfse_or_raw(&DependenciesImpl, raw), raw);
        let stream = DependenciesImpl.compress_lzfse(raw);
        assert_eq!(decompress_lzfse_or_raw(&DependenciesImpl, &stream), raw);
    }

    #[test]
    fn png_round_trips_pixels() {
        let image = VMaxImage {
            width: 2,
            height: 2,
            pixels: vec![[1, 2, 3, 255], [4, 5, 6, 0], [7, 8, 9, 128], [0, 0, 0, 1]],
        };
        let bytes = DependenciesImpl.encode_png(&image).unwrap();
        assert_eq!(DependenciesImpl.decode_png(&bytes).unwrap(), image);
    }

    #[test]
    fn palette_png_round_trips_as_a_strip() {
        let file = VMaxPalettePngFile(vec![[1, 2, 3, 255], [4, 5, 6, 255], [7, 8, 9, 0]]);
        let bytes = DependenciesImpl.encode_palette_png(&file).unwrap();
        let image = DependenciesImpl.decode_png(&bytes).unwrap();
        assert_eq!((image.width, image.height), (3, 1));
        assert_eq!(image.pixels, file.0);
    }

    #[test]
    fn png_rejects_a_zero_sized_image() {
        assert!(DependenciesImpl.encode_png(&VMaxImage::default()).is_err());
        assert!(DependenciesImpl.decode_png(b"not a png").is_err());
    }

    #[test]
    fn scene_json_round_trips_compact() {
        let scene = VMaxSceneJsonFile {
            v: 4,
            aint: Some(0.30000000000000004),
            ..Default::default()
        };
        let bytes = DependenciesImpl.encode_vmax_scene_json(&scene);
        assert!(!bytes.contains(&b'\n'));
        assert_eq!(
            DependenciesImpl.decode_vmax_scene_json(&bytes).unwrap(),
            scene
        );
        assert!(
            DependenciesImpl
                .decode_vmax_scene_json(b"not a scene")
                .is_err()
        );
    }
}
