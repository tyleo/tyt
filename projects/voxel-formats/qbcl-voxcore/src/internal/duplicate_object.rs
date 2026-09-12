use crate::{Error, Result};
use voxcore::{VoxMain, VoxObject};

/// A fresh object with `object`'s name, grid, origin, layers, and live
/// voxels, so a format that places one grid per object can give each extra
/// placement an object. A layer's default material, which only the empty
/// cells hold, is the material its first live voxel samples, or the palette's
/// first material for a layer with no live voxel. Errors when such a layer's
/// palette has no material, because the copy's empty cells need one.
pub fn duplicate_object(main: &VoxMain<()>, object: &VoxObject) -> Result<VoxObject> {
    let mut copy = VoxObject::new(object.name().to_owned(), object.bounds())
        .expect("the source object's grid is within the dense limit");
    copy.set_origin(object.origin());

    let layer_ids: Vec<_> = object.iter_layers().collect();
    let first_live = object.iter_live().next();
    for &(layer_id, palette_id) in &layer_ids {
        let sampled_id = first_live.and_then(|voxel_id| object.voxel_material(voxel_id, layer_id));
        let default_material_id = match sampled_id {
            Some(material_id) => material_id,
            None => {
                let palette = main
                    .palette(palette_id)
                    .expect("a layer references a live palette");
                let Some(material_id) = palette.iter_materials().next() else {
                    return Err(Error::Invalid(format!(
                        "object {} has no live voxel and its palette {palette_id} has no \
                         material to give the copy's empty cells",
                        object.name()
                    )));
                };
                material_id
            }
        };
        copy.retain_layer(palette_id, default_material_id);
    }

    let mut sample_ids = Vec::with_capacity(layer_ids.len());
    for voxel_id in object.iter_live() {
        sample_ids.clear();
        sample_ids.extend(layer_ids.iter().map(|&(layer_id, _)| {
            object
                .voxel_material(voxel_id, layer_id)
                .expect("a live voxel samples every layer")
        }));
        copy.retain_voxel(voxel_id, &sample_ids)
            .expect("the copy has the source's grid and layers");
    }

    Ok(copy)
}

#[cfg(test)]
mod tests {
    use crate::{Error, duplicate_object};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{VoxMain, VoxObject, VoxPalette, VoxValuePool};

    fn empty_object(main: &VoxMain<()>) -> VoxObject {
        let palette_id = main.iter_palettes().next().unwrap().0;

        let mut object = VoxObject::new("empty".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();

        object.retain_layer(palette_id, U32Id::from_u32(0));

        object
    }

    #[test]
    fn an_empty_layer_defaults_to_the_palettes_first_material() {
        let mut main: VoxMain = VoxMain::default();

        let value_pool_id = main.retain_value_pool(VoxValuePool::int(vec![1]).unwrap());

        let mut palette = VoxPalette::default();

        palette
            .retain_property("v".to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();

        palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();

        main.retain_palette(palette).unwrap();

        let copy = duplicate_object(&main, &empty_object(&main)).unwrap();

        assert_eq!(copy.layer_count(), 1);

        main.retain_object(copy).unwrap();

        main.validate().unwrap();
    }

    #[test]
    fn an_empty_layer_over_an_empty_palette_errors() {
        let mut main: VoxMain = VoxMain::default();

        main.retain_palette(VoxPalette::default()).unwrap();

        let actual = duplicate_object(&main, &empty_object(&main));

        assert!(matches!(actual, Err(Error::Invalid(_))));
    }
}
