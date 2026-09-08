use crate::{Result, ext::GoxlVoxMain, write_goxl};
use goxl::GoxlFile;

/// Writes a [`GoxlVoxMain`] back to a Goxel [`GoxlFile`], the inverse of
/// [`from_goxl_file_with_ext`](crate::ext::from_goxl_file_with_ext) and the
/// typed form of [`to_goxl_file`](crate::to_goxl_file). A loaded file writes
/// back exactly through its ext. A state carrying none writes a synthesized
/// file.
pub fn to_goxl_file_with_ext(state: &GoxlVoxMain) -> Result<GoxlFile> {
    write_goxl(state, state.ext().as_ref())
}

#[cfg(test)]
mod tests {
    use crate::ext::{from_goxl_file_with_ext, to_goxl_file_with_ext};
    use goxl::{
        GoxlBlock, GoxlCamera, GoxlDict, GoxlFile, GoxlImage, GoxlLayer, GoxlLayerBlock, GoxlLight,
        GoxlMaterial, GoxlPreview, GoxlShape, GoxlUnknownChunk, GoxlVoxel,
    };

    /// A `4 x 4` matrix with distinct float cells, for transform and box
    /// fields.
    fn matrix(base: f32) -> [[f32; 4]; 4] {
        let mut matrix = [[0.0f32; 4]; 4];
        for (index, cell) in matrix.iter_mut().flatten().enumerate() {
            *cell = base + index as f32 * 0.5;
        }
        matrix
    }

    /// A full `16 x 16 x 16` block: mostly empty (all-zero cells), with a few
    /// tagged voxels including one with a partial alpha.
    fn block() -> GoxlBlock {
        let mut voxels = vec![GoxlVoxel::default(); GoxlBlock::SIZE.pow(3) as usize];
        voxels[0] = GoxlVoxel::new(10, 20, 30);
        voxels[1] = GoxlVoxel::new(255, 0, 0);
        voxels[100] = GoxlVoxel {
            r: 1,
            g: 2,
            b: 3,
            a: 128,
        };
        *voxels.last_mut().unwrap() = GoxlVoxel::new(7, 8, 9);
        GoxlBlock { voxels }
    }

    /// A pair whose value is raw bytes, for `extra` dictionaries.
    fn extra(key: &str, value: &[u8]) -> (String, Vec<u8>) {
        (key.to_owned(), value.to_vec())
    }

    /// A file exercising every modeled chunk: two shared blocks, a normal layer
    /// stamping both, a clone layer, a shape layer, materials, cameras, a
    /// light, a preview, an image box, and an unknown chunk.
    fn sample_file() -> GoxlFile {
        GoxlFile {
            version: 2,
            image: GoxlImage {
                bounding_box: Some(matrix(1.0)),
                extra: GoxlDict(vec![extra("vendor", &[1, 2, 3, 4])]),
            },
            preview: Some(GoxlPreview {
                width: 2,
                height: 3,
                pixels: vec![
                    [0, 0, 0, 0],
                    [255, 255, 255, 255],
                    [1, 2, 3, 4],
                    [5, 6, 7, 8],
                    [9, 10, 11, 12],
                    [13, 14, 15, 16],
                ],
            }),
            blocks: vec![
                block(),
                GoxlBlock {
                    voxels: vec![GoxlVoxel::new(50, 60, 70); GoxlBlock::SIZE.pow(3) as usize],
                },
            ],
            materials: vec![
                GoxlMaterial {
                    name: "Default".to_owned(),
                    base_color: [0.25, 0.5, 0.75, 1.0],
                    metallic: 0.1,
                    roughness: 0.9,
                    emission: [0.0, 0.5, 1.0],
                    extra: GoxlDict(vec![extra("_custom", &[9])]),
                },
                GoxlMaterial::default(),
            ],
            layers: vec![
                GoxlLayer {
                    name: "Layer 0".to_owned(),
                    id: 1,
                    base_id: 0,
                    material: 0,
                    mode: 0,
                    visible: true,
                    transform: matrix(2.0),
                    blocks: vec![
                        GoxlLayerBlock {
                            block_index: 0,
                            position: [0, 0, 0],
                        },
                        GoxlLayerBlock {
                            block_index: 1,
                            position: [-16, 16, -32],
                        },
                    ],
                    bounding_box: Some(matrix(3.0)),
                    image_path: Some("/tmp/ref.png".to_owned()),
                    shape: None,
                    color: None,
                    extra: GoxlDict(vec![extra("_layer", &[7, 7])]),
                },
                GoxlLayer {
                    name: "Clone".to_owned(),
                    id: 2,
                    base_id: 1,
                    material: 1,
                    mode: 1,
                    visible: false,
                    transform: matrix(4.0),
                    blocks: Vec::new(),
                    bounding_box: None,
                    image_path: None,
                    shape: None,
                    color: None,
                    extra: GoxlDict::default(),
                },
                GoxlLayer {
                    name: "Shape".to_owned(),
                    id: 3,
                    base_id: 0,
                    material: 0,
                    mode: 0,
                    visible: true,
                    transform: matrix(5.0),
                    blocks: Vec::new(),
                    bounding_box: None,
                    image_path: None,
                    shape: Some(GoxlShape::Cylinder),
                    color: Some([200, 150, 100, 255]),
                    extra: GoxlDict::default(),
                },
            ],
            cameras: vec![
                GoxlCamera {
                    name: "Camera".to_owned(),
                    distance: 12.5,
                    orthographic: true,
                    transform: matrix(6.0),
                    active: true,
                    extra: GoxlDict(vec![extra("_cam", &[3, 3, 3])]),
                },
                GoxlCamera::default(),
            ],
            light: Some(GoxlLight {
                pitch: 0.5,
                yaw: -0.25,
                intensity: 1.5,
                fixed: true,
                ambient: 0.2,
                shadow: 0.8,
                extra: GoxlDict(vec![extra("_light", &[1])]),
            }),
            unknown_chunks: vec![GoxlUnknownChunk {
                id: *b"XTRA",
                data: vec![9, 8, 7, 6, 5],
            }],
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let file = sample_file();
        let state = from_goxl_file_with_ext(&file).unwrap();
        assert_eq!(to_goxl_file_with_ext(&state).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = GoxlFile::default();
        let state = from_goxl_file_with_ext(&file).unwrap();
        assert_eq!(to_goxl_file_with_ext(&state).unwrap(), file);
    }

    /// A layer may stamp the same block at several positions. The voxcore node
    /// lists the placed object once, but the ext keeps the full placement list,
    /// so the layer round-trips.
    #[test]
    fn round_trips_a_layer_stamping_one_block_twice() {
        let file = GoxlFile {
            blocks: vec![block()],
            layers: vec![GoxlLayer {
                name: "L".to_owned(),
                blocks: vec![
                    GoxlLayerBlock {
                        block_index: 0,
                        position: [0, 0, 0],
                    },
                    GoxlLayerBlock {
                        block_index: 0,
                        position: [16, 0, 0],
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        };
        let state = from_goxl_file_with_ext(&file).unwrap();
        assert_eq!(to_goxl_file_with_ext(&state).unwrap(), file);
    }

    #[cfg(feature = "codec")]
    mod codec {
        use super::*;
        use crate::codec::{from_goxl_bytes_with_ext, to_goxl_bytes_with_ext};
        use goxl_codec::DependenciesImpl;

        #[test]
        fn round_trips_through_goxl_bytes() {
            let file = sample_file();
            let state = from_goxl_file_with_ext(&file).unwrap();
            let bytes = to_goxl_bytes_with_ext(&DependenciesImpl, &state).unwrap();
            let reloaded = from_goxl_bytes_with_ext(&DependenciesImpl, &bytes).unwrap();
            assert_eq!(to_goxl_file_with_ext(&reloaded).unwrap(), file);
        }
    }
}
