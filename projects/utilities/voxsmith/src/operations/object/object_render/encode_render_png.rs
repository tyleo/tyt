use crate::{
    Error, Result,
    dependencies::object::{EncodePng, PngChannels, PngImage},
    operations::object::Transfer,
};
use voxrender::RenderOutput;

/// Encodes `image` as an 8-bit RGBA PNG with the sRGB transfer stamped.
pub fn encode_render_png<D: EncodePng>(dependencies: &D, image: &RenderOutput) -> Result<Vec<u8>> {
    dependencies
        .encode_png(&PngImage {
            width: image.width(),
            height: image.height(),
            channels: PngChannels::Rgba,
            transfer: Transfer::Srgb,
            samples: image.to_bytes(),
        })
        .map_err(Error::Png)
}

#[cfg(test)]
mod tests {
    use crate::{
        Error,
        dependencies::{
            DependenciesImpl,
            object::{EncodePng, PngImage},
        },
        operations::object::{
            FitOrFixed, LightRecord, PoseTransform, RenderRecord, Rotation, RotationTransform,
            ViewProjection, ViewRecord, encode_render_png, render,
        },
        test_utilities::two_material_scene,
    };
    use branded_id::{IdVec, U32Id};
    use png::{ColorType, Decoder, Info, SrgbRenderingIntent};
    use std::{env, fs, io::Cursor, path::Path, result::Result as StdResult};
    use ty_math::{TyLinSrgbF64, TySrgbU8, TyVector3I32, TyVector3U32};
    use voxrender::{RenderBloom, RenderImage, RenderOcclusion, RenderOutput, RenderShadow};

    /// Refuses every image.
    struct Refuse;

    impl EncodePng for Refuse {
        fn encode_png(&self, _: &PngImage) -> StdResult<Vec<u8>, String> {
            Err("no".to_owned())
        }
    }

    /// The samples and header of the PNG `bytes`.
    fn decode(bytes: &[u8]) -> (Vec<u8>, Info<'static>) {
        let mut reader = Decoder::new(Cursor::new(bytes)).read_info().unwrap();
        let mut buffer = vec![0u8; reader.output_buffer_size().unwrap()];
        let output = reader.next_frame(&mut buffer).unwrap();
        buffer.truncate(output.buffer_size());
        (buffer, reader.info().clone())
    }

    #[test]
    fn the_png_holds_the_pixels_with_the_srgb_transfer_stamped() {
        let image = RenderOutput::from_image(&RenderImage::new(2, 1), Some(TySrgbU8::new(1, 2, 3)));

        let png = encode_render_png(&DependenciesImpl, &image).unwrap();

        let (samples, info) = decode(&png);
        assert_eq!((info.width, info.height), (2, 1));
        assert_eq!(info.color_type, ColorType::Rgba);
        assert_eq!(info.srgb, Some(SrgbRenderingIntent::Perceptual));
        assert_eq!(samples, [1, 2, 3, 255, 1, 2, 3, 255]);
    }

    #[test]
    fn a_refused_png_errors() {
        let image = RenderOutput::from_image(&RenderImage::new(1, 1), None);

        assert!(matches!(
            encode_render_png(&Refuse, &image),
            Err(Error::Png(_))
        ));
    }

    /// `VOXSMITH_UPDATE_GOLDENS` rewrites the golden beside this file.
    #[test]
    fn the_two_material_scene_matches_its_golden_under_hero_and_studio() {
        let main = two_material_scene(
            TyVector3U32::new(3, 2, 2),
            TyVector3I32::ZERO,
            &[
                (TyVector3U32::new(0, 0, 0), 0),
                (TyVector3U32::new(1, 0, 0), 0),
                (TyVector3U32::new(2, 0, 0), 1),
                (TyVector3U32::new(0, 0, 1), 1),
                (TyVector3U32::new(1, 1, 0), 1),
            ],
        );

        let studio = vec![
            LightRecord::Directional {
                transform: RotationTransform::Camera {
                    rotation: Rotation::Angles {
                        azimuth: -30.0,
                        elevation: 30.0,
                    },
                },
                shadow: RenderShadow::PerCorner,
                color: TyLinSrgbF64::new(1.0, 1.0, 1.0),
                strength: 2.5,
            },
            LightRecord::Hemisphere {
                sky: TySrgbU8::new(0x9F, 0xB4, 0xCC)
                    .into_format::<f64>()
                    .into_linear(),
                ground: TySrgbU8::new(0x4A, 0x3E, 0x33)
                    .into_format::<f64>()
                    .into_linear(),
                strength: 1.0,
            },
        ];

        let record = RenderRecord {
            width: 32,
            height: 32,
            background: None,
            occlusion: RenderOcclusion::Corner,
            voxel_size: 1.0,
            bloom: RenderBloom::default(),
            views: IdVec::from(vec![ViewRecord {
                name: "hero".to_owned(),
                transform: PoseTransform::Orbit {
                    azimuth: 45.0,
                    elevation: 30.0,
                    distance: FitOrFixed::Fit,
                },
                projection: ViewProjection::Perspective { fov: 35.0 },
                select: Vec::new(),
            }]),
            lights: IdVec::from(studio),
        };

        let outputs = render(&main, &[U32Id::from_u32(0)], &record).unwrap();
        let png = encode_render_png(&DependenciesImpl, &outputs.as_slice()[0].image).unwrap();

        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/operations/object/object_render/goldens/two-material-hero-studio.png");

        if env::var_os("VOXSMITH_UPDATE_GOLDENS").is_some() {
            fs::write(&path, &png).unwrap();
            return;
        }

        let golden: &[u8] = include_bytes!("goldens/two-material-hero-studio.png");
        assert!(
            !golden.is_empty(),
            "no golden; run with VOXSMITH_UPDATE_GOLDENS=1 to write it"
        );

        let (expected, expected_info) = decode(golden);
        let (actual, info) = decode(&png);
        assert_eq!(info.srgb, expected_info.srgb);
        assert_eq!((info.width, info.height), (32, 32));
        assert_eq!(expected.len(), actual.len());

        let strays = expected
            .iter()
            .zip(&actual)
            .filter(|(expected, actual)| expected.abs_diff(**actual) > 2)
            .count();
        assert_eq!(strays, 0, "{strays} channels stray past 2");
    }
}
