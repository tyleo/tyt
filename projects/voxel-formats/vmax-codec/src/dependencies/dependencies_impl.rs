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
    fit_object_refs(bytes)
}

/// Rewrites a binary plist so Apple's reader accepts its object references.
/// CoreFoundation rejects a binary plist whose object count reaches
/// `2^(8 * width)` for its reference width, while the `plist` crate sizes the
/// width for the highest object index. A plist of exactly 256 objects then
/// carries 1-byte references, and Voxel Max fails to open the file. Widens the
/// references of such a plist and leaves any other untouched. Every object
/// keeps its bytes but its references.
fn fit_object_refs(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    let trailer = bytes
        .len()
        .checked_sub(32)
        .ok_or("a binary plist ends in a 32-byte trailer")?;
    let offset_width = usize::from(bytes[trailer + 6]);
    let ref_width = usize::from(bytes[trailer + 7]);
    let count = be_uint(&bytes[trailer + 8..trailer + 16]);
    if ref_width >= 8 || count < 1 << (8 * ref_width) {
        return Ok(bytes);
    }
    let table = be_uint(&bytes[trailer + 24..trailer + 32]) as usize;
    let wide = uint_width(count);

    let mut out = b"bplist00".to_vec();
    let mut offsets = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let at = table + index * offset_width;
        let start = be_uint(&bytes[at..at + offset_width]) as usize;
        offsets.push(out.len() as u64);
        let kind = bytes[start] >> 4;
        let (length, header_end) = object_length(&bytes, start)?;
        match kind {
            0xA | 0xC | 0xD => {
                out.extend_from_slice(&bytes[start..header_end]);
                let refs = if kind == 0xD { 2 * length } else { length };
                for slot in 0..refs {
                    let at = header_end + slot * ref_width;
                    let object = be_uint(&bytes[at..at + ref_width]);
                    out.extend_from_slice(&object.to_be_bytes()[8 - wide..]);
                }
            }

            _ => out.extend_from_slice(&bytes[start..header_end + length]),
        }
    }

    let table = out.len() as u64;
    let offset_width = uint_width(table);
    for offset in offsets {
        out.extend_from_slice(&offset.to_be_bytes()[8 - offset_width..]);
    }
    out.extend_from_slice(&[0; 6]);
    out.extend_from_slice(&[offset_width as u8, wide as u8]);
    out.extend_from_slice(&count.to_be_bytes());
    out.extend_from_slice(&bytes[trailer + 16..trailer + 24]);
    out.extend_from_slice(&table.to_be_bytes());
    Ok(out)
}

/// The object at `start` as `(length, header_end)`: a container's entry count
/// or any other object's payload length in bytes, and where its marker and
/// count end.
fn object_length(bytes: &[u8], start: usize) -> Result<(usize, usize), String> {
    let marker = bytes[start];
    let low = usize::from(marker & 0x0f);
    let counted = || -> (usize, usize) {
        if low < 0x0f {
            return (low, start + 1);
        }
        let width = 1 << (bytes[start + 1] & 0x0f);
        let count = be_uint(&bytes[start + 2..start + 2 + width]) as usize;
        (count, start + 2 + width)
    };
    Ok(match marker >> 4 {
        0x0 => (0, start + 1),
        0x1 | 0x2 => (1 << low, start + 1),
        0x3 => (8, start + 1),
        0x4 | 0x5 | 0x7 | 0xA | 0xC | 0xD => counted(),
        0x6 => {
            let (count, end) = counted();
            (2 * count, end)
        }

        0x8 => (low + 1, start + 1),
        kind => return Err(format!("binary plist object marker {kind:#x} is unknown")),
    })
}

/// The big-endian unsigned integer in `bytes`.
fn be_uint(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .fold(0, |value, &byte| (value << 8) | u64::from(byte))
}

/// The narrowest plist integer width, 1, 2, 4, or 8 bytes, whose range holds
/// `value` below its limit, as CoreFoundation requires of reference and offset
/// widths.
fn uint_width(value: u64) -> usize {
    [1, 2, 4]
        .into_iter()
        .find(|&width| value < 1 << (8 * width))
        .unwrap_or(8)
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
    fn encode_vmax_scene_json(&self, file: &VMaxSceneJsonFile) -> Result<Vec<u8>, String> {
        serde_json::to_vec(file).map_err(|error| error.to_string())
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
        CompressLzfse, DecodePng, DecodeVMaxPlist, DecodeVMaxSceneJson, DecompressLzfse,
        DependenciesImpl, EncodePng, EncodeVMaxPlist, EncodeVMaxSceneJson,
    };
    use std::collections::BTreeMap;
    use vmax::{
        VMaxHistoryVmaxhvscFile, VMaxImage, VMaxPalettePngFile, VMaxSceneJsonFile, VMaxValue,
    };

    /// A binary plist's object count and reference width, from its trailer.
    fn objects_and_ref_width(bytes: &[u8]) -> (u64, u8) {
        let trailer = &bytes[bytes.len() - 32..];
        let count = trailer[8..16]
            .iter()
            .fold(0, |value, &byte| (value << 8) | u64::from(byte));
        (count, trailer[7])
    }

    /// Encodes `file`, checks it holds `objects` objects at a reference width
    /// CoreFoundation reads, and decodes it back.
    fn encode_objects(file: &VMaxHistoryVmaxhvscFile, objects: u64) -> Vec<u8> {
        let bytes = DependenciesImpl.encode_history_vmaxhvsc(file).unwrap();
        let (count, width) = objects_and_ref_width(&bytes);
        assert_eq!(count, objects);
        assert!(
            count < 1 << (8 * u32::from(width)),
            "{count} objects at {width}-byte refs"
        );
        assert_eq!(
            &DependenciesImpl.decode_history_vmaxhvsc(&bytes).unwrap(),
            file
        );
        bytes
    }

    /// The cache and pivot keys a current Voxel Max writes on objects and
    /// groups parse into their fields.
    #[test]
    fn a_scene_from_a_current_voxel_max_parses() {
        let json = br#"{"v":4,"objects":[{"id":"o","data":"contents.vmaxb","hist":"history.vmaxhb","pal":"palette.png","t_p":[0,0,0],"t_r":[0,0,0,0],"t_s":[1,1,1],"ind":[0,0,0],"e_c":[1,1,1],"e_mi":[-1,-1,-1],"e_ma":[1,1,1],"e_cm":[1,1,1],"e_cmv":2,"e_vc":8,"e_vm":8.5,"t_prp":[0.5,0.5,0.5]}],"groups":[{"id":"g","name":"group","t_p":[0,0,0],"t_r":[0,0,0,0],"t_s":[1,1,1],"ind":[0,1,0],"e_c":[0,0,0],"e_cm":[0,0,0],"e_cmv":2,"e_vc":8}]}"#;

        let scene = DependenciesImpl.decode_vmax_scene_json(json).unwrap();

        let object = &scene.objects[0];
        assert_eq!(
            (object.e_vc, object.e_vm, object.e_cmv, object.t_prp),
            (Some(8), Some(8.5), Some(2), Some([0.5, 0.5, 0.5]))
        );
        assert_eq!(scene.groups[0].e_vc, Some(8));
    }

    /// An array of exactly 256 objects takes 2-byte references, since Apple's
    /// reader, and so Voxel Max, rejects 1-byte references to 256 objects.
    #[test]
    fn a_plist_of_256_objects_takes_references_apple_reads() {
        let file = VMaxHistoryVmaxhvscFile((0..255).map(VMaxValue::Integer).collect());
        encode_objects(&file, 256);
    }

    /// A dictionary's key and value references widen with the array's.
    #[test]
    fn a_dictionary_at_the_reference_limit_widens_its_references() {
        let entries = (0..127)
            .map(|index| (format!("key{index:03}"), VMaxValue::Integer(1000 + index)))
            .collect::<BTreeMap<_, _>>();
        let file = VMaxHistoryVmaxhvscFile(vec![VMaxValue::Dictionary(entries)]);
        encode_objects(&file, 256);
    }

    /// A plist under the reference limit keeps the bytes the `plist` crate
    /// writes.
    #[test]
    fn a_plist_under_the_reference_limit_keeps_its_bytes() {
        let file = VMaxHistoryVmaxhvscFile((0..254).map(VMaxValue::Integer).collect());
        let bytes = encode_objects(&file, 255);
        let mut plain = Vec::new();
        plist::to_writer_binary(&mut plain, &file).unwrap();
        assert_eq!(bytes, plain);
    }

    #[test]
    fn lzfse_round_trips_and_frames_the_stream() {
        let bytes: Vec<u8> = (0..4096u32).map(|i| (i % 7) as u8).collect();
        let stream = DependenciesImpl.compress_lzfse(&bytes);
        assert!(stream.starts_with(b"bvx"));
        assert!(stream.len() < bytes.len());
        assert_eq!(DependenciesImpl.decompress_lzfse(&stream).unwrap(), bytes);
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
        let bytes = DependenciesImpl.encode_vmax_scene_json(&scene).unwrap();
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
