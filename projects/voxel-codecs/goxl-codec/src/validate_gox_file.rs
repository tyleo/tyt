use crate::{Result, invalid};
use goxl::{GoxlBlock, GoxlDict, GoxlFile};
use std::collections::HashSet;

/// The number of voxels in a `BL16` block: [`GoxlBlock::SIZE`]`^3 == 4096`.
const BLOCK_VOXELS: usize = (GoxlBlock::SIZE * GoxlBlock::SIZE * GoxlBlock::SIZE) as usize;

/// Checks a decoded [`GoxlFile`] for shape and cross-reference faults the `.gox`
/// byte layout cannot catch:
///
/// 1. every block holds exactly [`GoxlBlock::SIZE`]`^3` voxels, so it encodes to
///    a `64 x 64` PNG;
/// 2. the preview has non-zero dimensions and holds exactly `width * height`
///    pixels;
/// 3. layer ids are unique (ignoring the unset id `0`), and a clone layer's
///    [`base_id`](goxl::GoxlLayer::base_id) resolves to an existing layer;
/// 4. a clone (`base_id != 0`) or shape layer carries no placed blocks, as Goxel
///    writes none for one;
/// 5. every layer's [`material`](goxl::GoxlLayer::material) indexes an existing
///    material, and every placed block indexes an existing block;
/// 6. no `extra` dictionary carries a key this crate models as a typed field on
///    the same chunk, which the encoder would emit twice and a re-decode would
///    silently fold back onto the typed field.
///
/// Decoding rejects malformed bytes and never places a modeled key in an `extra`
/// dictionary. A hand-built or edited [`GoxlFile`] can still hold mis-sized
/// arrays, dangling references, or colliding `extra` keys that
/// [`to_gox_file_bytes`](crate::to_gox_file_bytes) would write as a structurally
/// valid but broken file. Decoding does not call this. Run it when you need the
/// guarantee.
pub fn validate_gox_file(file: &GoxlFile) -> Result<()> {
    validate_blocks(file)?;
    validate_preview(file)?;
    validate_layers(file)?;
    validate_extras(file)?;
    Ok(())
}

/// Every block holds exactly one full `16 x 16 x 16` grid.
fn validate_blocks(file: &GoxlFile) -> Result<()> {
    for (index, block) in file.blocks.iter().enumerate() {
        if block.voxels.len() != BLOCK_VOXELS {
            return Err(invalid(format!(
                "block {index} holds {} voxels, expected {BLOCK_VOXELS}",
                block.voxels.len()
            )));
        }
    }
    Ok(())
}

/// The preview has non-zero dimensions and holds exactly `width * height` pixels.
fn validate_preview(file: &GoxlFile) -> Result<()> {
    if let Some(preview) = &file.preview {
        if preview.width == 0 || preview.height == 0 {
            return Err(invalid(format!(
                "preview has a zero dimension ({}x{}); a PNG preview needs positive width and height",
                preview.width, preview.height
            )));
        }
        // `u64` so the product cannot overflow for large `u32` dimensions.
        let expected = preview.width as u64 * preview.height as u64;
        if preview.pixels.len() as u64 != expected {
            return Err(invalid(format!(
                "preview is {}x{} = {expected} pixels but holds {}",
                preview.width,
                preview.height,
                preview.pixels.len()
            )));
        }
    }
    Ok(())
}

/// Layer ids are unique, references resolve, and clone / shape layers are empty.
fn validate_layers(file: &GoxlFile) -> Result<()> {
    // The unset id 0 is allowed to repeat; any real id must be unique.
    let mut ids = HashSet::with_capacity(file.layers.len());
    for layer in &file.layers {
        if layer.id != 0 && !ids.insert(layer.id) {
            return Err(invalid(format!(
                "layer id {} is declared more than once",
                layer.id
            )));
        }
    }

    for (index, layer) in file.layers.iter().enumerate() {
        if layer.material < 0 || layer.material as usize >= file.materials.len() {
            return Err(invalid(format!(
                "layer {index} references material {}, but the file has {} materials",
                layer.material,
                file.materials.len()
            )));
        }
        if layer.base_id != 0 && !ids.contains(&layer.base_id) {
            return Err(invalid(format!(
                "layer {index} clones base_id {}, which no layer declares",
                layer.base_id
            )));
        }
        if (layer.base_id != 0 || layer.shape.is_some()) && !layer.blocks.is_empty() {
            return Err(invalid(format!(
                "layer {index} is a clone or shape layer but carries {} placed blocks",
                layer.blocks.len()
            )));
        }
        for placed in &layer.blocks {
            if placed.block_index < 0 || placed.block_index as usize >= file.blocks.len() {
                return Err(invalid(format!(
                    "layer {index} places block index {}, but the file has {} blocks",
                    placed.block_index,
                    file.blocks.len()
                )));
            }
        }
    }
    Ok(())
}

/// No `extra` dictionary carries a key this crate models as a typed field on the
/// same chunk. The decoder never produces such a collision; a hand-built one
/// would make the encoder write the key twice (once from the field, once from
/// `extra`) and a re-decode would fold the value back onto the typed field,
/// silently dropping the `extra` copy.
fn validate_extras(file: &GoxlFile) -> Result<()> {
    check_extra("image", &file.image.extra, &["box"])?;
    for (index, material) in file.materials.iter().enumerate() {
        check_extra(
            &format!("material {index}"),
            &material.extra,
            &["name", "color", "metallic", "roughness", "emission"],
        )?;
    }
    for (index, layer) in file.layers.iter().enumerate() {
        check_extra(
            &format!("layer {index}"),
            &layer.extra,
            &[
                "name", "mat", "id", "base_id", "material", "mode", "img-path", "box", "shape",
                "color", "visible",
            ],
        )?;
    }
    for (index, camera) in file.cameras.iter().enumerate() {
        check_extra(
            &format!("camera {index}"),
            &camera.extra,
            &["name", "dist", "ortho", "mat", "active"],
        )?;
    }
    if let Some(light) = &file.light {
        check_extra(
            "light",
            &light.extra,
            &["pitch", "yaw", "intensity", "fixed", "ambient", "shadow"],
        )?;
    }
    Ok(())
}

/// Rejects an `extra` dictionary that carries one of `modeled`, the keys the
/// owning chunk lifts onto typed fields.
fn check_extra(owner: &str, extra: &GoxlDict, modeled: &[&str]) -> Result<()> {
    for (key, _) in &extra.0 {
        if modeled.contains(&key.as_str()) {
            return Err(invalid(format!(
                "{owner} extra dictionary holds the modeled key {key:?}, which its typed field owns"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::validate_gox_file;
    use goxl::{
        GoxlBlock, GoxlDict, GoxlFile, GoxlLayer, GoxlLayerBlock, GoxlMaterial, GoxlPreview,
        GoxlShape, GoxlVoxel,
    };

    /// A full block of empty voxels.
    fn block() -> GoxlBlock {
        GoxlBlock {
            voxels: vec![GoxlVoxel::default(); super::BLOCK_VOXELS],
        }
    }

    /// A small valid file: one block, one material, one layer placing the block.
    fn valid_file() -> GoxlFile {
        GoxlFile {
            blocks: vec![block()],
            materials: vec![GoxlMaterial::default()],
            layers: vec![GoxlLayer {
                id: 1,
                material: 0,
                blocks: vec![GoxlLayerBlock {
                    block_index: 0,
                    position: [0, 0, 0],
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn accepts_a_valid_file() {
        assert!(validate_gox_file(&valid_file()).is_ok());
    }

    #[test]
    fn accepts_a_default_file() {
        assert!(validate_gox_file(&GoxlFile::default()).is_ok());
    }

    #[test]
    fn rejects_a_mis_sized_block() {
        let mut file = valid_file();
        file.blocks[0].voxels.pop();
        assert!(validate_gox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_mis_sized_preview() {
        let mut file = valid_file();
        file.preview = Some(GoxlPreview {
            width: 2,
            height: 2,
            pixels: vec![[0, 0, 0, 0]],
        });
        assert!(validate_gox_file(&file).is_err());
    }

    #[test]
    fn rejects_an_oversized_preview() {
        let mut file = valid_file();
        // A pixel count no real buffer can match is caught by the count check
        // without overflowing.
        file.preview = Some(GoxlPreview {
            width: u32::MAX,
            height: u32::MAX,
            pixels: vec![[0, 0, 0, 0]],
        });
        assert!(validate_gox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_modeled_key_in_extra() {
        let mut file = valid_file();
        // "metallic" is a typed MATE field, so it must not appear in `extra`.
        file.materials[0].extra = GoxlDict(vec![("metallic".to_owned(), vec![0, 0, 0, 0])]);
        assert!(validate_gox_file(&file).is_err());
    }

    #[test]
    fn accepts_an_unmodeled_key_in_extra() {
        let mut file = valid_file();
        file.materials[0].extra = GoxlDict(vec![("vendor".to_owned(), vec![1, 2, 3])]);
        assert!(validate_gox_file(&file).is_ok());
    }

    #[test]
    fn rejects_a_zero_dimension_preview() {
        let mut file = valid_file();
        file.preview = Some(GoxlPreview {
            width: 0,
            height: 0,
            pixels: Vec::new(),
        });
        assert!(validate_gox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_duplicate_layer_id() {
        let mut file = valid_file();
        file.layers.push(GoxlLayer {
            id: 1,
            material: 0,
            ..Default::default()
        });
        assert!(validate_gox_file(&file).is_err());
    }

    #[test]
    fn allows_repeated_unset_layer_ids() {
        let mut file = valid_file();
        file.layers[0].id = 0;
        file.layers.push(GoxlLayer {
            id: 0,
            material: 0,
            ..Default::default()
        });
        assert!(validate_gox_file(&file).is_ok());
    }

    #[test]
    fn rejects_an_out_of_range_material() {
        let mut file = valid_file();
        file.layers[0].material = 5;
        assert!(validate_gox_file(&file).is_err());
    }

    #[test]
    fn rejects_a_dangling_base_id() {
        let mut file = valid_file();
        file.layers[0].blocks.clear();
        file.layers[0].base_id = 99;
        assert!(validate_gox_file(&file).is_err());
    }

    #[test]
    fn rejects_blocks_on_a_clone_layer() {
        let mut file = valid_file();
        // The layer places a block; cloning another id makes that illegal.
        file.layers.push(GoxlLayer {
            id: 2,
            base_id: 1,
            material: 0,
            blocks: vec![GoxlLayerBlock {
                block_index: 0,
                position: [0, 0, 0],
            }],
            ..Default::default()
        });
        assert!(validate_gox_file(&file).is_err());
    }

    #[test]
    fn rejects_blocks_on_a_shape_layer() {
        let mut file = valid_file();
        file.layers[0].shape = Some(GoxlShape::Cube);
        assert!(validate_gox_file(&file).is_err());
    }

    #[test]
    fn rejects_an_out_of_range_block_index() {
        let mut file = valid_file();
        file.layers[0].blocks[0].block_index = 9;
        assert!(validate_gox_file(&file).is_err());
    }
}
