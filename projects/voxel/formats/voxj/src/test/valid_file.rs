use crate::{
    VoxjFile, VoxjMain, VoxjObject, VoxjPositionBlock, VoxjRuntimeState, VoxjSampleBlock,
    VoxjValuePool,
    test::{node, palette},
};

/// A small but complete valid document: one four-material palette over a
/// single color value pool, an object sampling it across two in-bounds voxels
/// (raw-json blocks), and a two-node DAG with a root.
pub fn valid_file() -> VoxjFile {
    VoxjFile {
        version: 1,
        main: VoxjMain {
            runtime_state: VoxjRuntimeState {
                value_pools: value_pools(),
                palettes: vec![palette(4)],
                objects: vec![VoxjObject {
                    name: "o".to_owned(),
                    layers: vec![0],
                    bounds: [2, 1, 1],
                    origin: [0, 0, 0],
                    voxel_positions: VoxjPositionBlock::RawJson(vec![[0, 0, 0], [1, 0, 0]]),
                    voxel_samples: VoxjSampleBlock::RawJson(vec![vec![1, 3]]),
                }],
                nodes: vec![node(vec![1], vec![0]), node(vec![], vec![])],
                root_nodes: vec![0],
            },
            edit_state: None,
            ext: None,
        },
    }
}

/// A `vec-4-float` value pool of four colors backing the property's
/// value-indices, and an unreferenced one-value `float` value pool.
fn value_pools() -> Vec<VoxjValuePool> {
    vec![
        VoxjValuePool::Vec4Float(vec![[0.0, 0.0, 0.0, 1.0]; 4]),
        VoxjValuePool::Float(vec![1.5]),
    ]
}
