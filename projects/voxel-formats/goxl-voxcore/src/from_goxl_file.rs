use crate::{Result, read_goxl};
use goxl::GoxlFile;
use voxcore::VoxMain;

/// Loads a Goxel [`GoxlFile`] into a bare [`VoxMain`]. The shared `BL16` voxel
/// blocks become objects sharing one `baseColor` palette, and the `LAYR` layers
/// become the hierarchy nodes placing them. The rest of the Goxel state is
/// dropped, so [`to_goxl_file`](crate::to_goxl_file) writes the state back as a
/// synthesized file.
/// [`ext::from_goxl_file_with_ext`](crate::ext::from_goxl_file_with_ext) keeps
/// that state instead.
///
/// Errors on a layer placement that references a block outside the block list,
/// or on a cross-reference the checked insertions reject.
pub fn from_goxl_file(file: &GoxlFile) -> Result<VoxMain<()>> {
    read_goxl(file)
}

#[cfg(test)]
mod tests {
    use crate::from_goxl_file;
    use goxl::{GoxlFile, GoxlLayer, GoxlLayerBlock};

    #[test]
    fn rejects_a_dangling_layer_block_placement() {
        let file = GoxlFile {
            layers: vec![GoxlLayer {
                blocks: vec![GoxlLayerBlock {
                    block_index: 5,
                    position: [0, 0, 0],
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(from_goxl_file(&file).is_err());
    }
}
