use crate::{Result, read_mvox};
use mvox::MVoxFile;
use voxcore::VoxMain;

/// Loads a decoded MagicaVoxel [`MVoxFile`] into a bare [`VoxMain`]. Models
/// become objects, the 256-color palette plus the `MATL` materials become one
/// shared palette of value pools, and the `nTRN` / `nGRP` / `nSHP` scene
/// graph becomes the hierarchy nodes. The rest of the MagicaVoxel state is
/// dropped, so [`to_mvox_file`](crate::to_mvox_file) writes the state back as
/// a synthesized file. The `ext` feature's `ext::from_mvox_file_with_ext`
/// keeps that state instead.
///
/// Errors if:
///
/// 1. the geometry is malformed
/// 2. a material id is outside the palette range
/// 3. a scene-node reference dangles
/// 4. a checked insertion rejects a cross-reference
pub fn from_mvox_file(file: &MVoxFile) -> Result<VoxMain<()>> {
    let (state, _) = read_mvox(file)?;

    Ok(state)
}

#[cfg(test)]
mod tests {
    use crate::from_mvox_file;
    use mvox::{
        MVoxFile, MVoxFrame, MVoxMaterial, MVoxModel, MVoxNodeAttributes, MVoxSceneNode,
        MVoxSceneNodeBody, MVoxTransformNode, MVoxVoxel,
    };

    #[test]
    fn rejects_a_material_id_outside_the_palette_range() {
        let file = MVoxFile {
            materials: vec![MVoxMaterial {
                id: 300,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(from_mvox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_dangling_scene_node_child() {
        let file = MVoxFile {
            scene_nodes: vec![MVoxSceneNode {
                id: 0,
                attributes: MVoxNodeAttributes::default(),
                body: MVoxSceneNodeBody::Transform(MVoxTransformNode {
                    child: 99,
                    layer: -1,
                    frames: vec![MVoxFrame::default()],
                }),
            }],
            ..Default::default()
        };
        assert!(from_mvox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_voxel_outside_its_model() {
        let file = MVoxFile {
            models: vec![MVoxModel {
                size: [1, 1, 1],
                voxels: vec![MVoxVoxel {
                    x: 5,
                    y: 0,
                    z: 0,
                    color_index: 1,
                }],
            }],
            ..Default::default()
        };
        assert!(from_mvox_file(&file).is_err());
    }
}
