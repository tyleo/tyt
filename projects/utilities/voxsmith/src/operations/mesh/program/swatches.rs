use crate::Result;
use branded_id::{IdVec, U32Id, UsizeId};
use std::collections::HashMap;
use vox_value_language::{BSwatch, BVoxel};
use voxcore::{
    BVoxEffectiveProperty, BVoxLayer, BVoxMaterial, BVoxVoxel, VoxEffectivePalette, VoxExt,
    VoxMain, VoxObject, VoxValuePoolValueRef,
};

/// An object's swatches: its distinct flattened materials in first-seen
/// raster order, one per palette-atlas texel. A voxel's flattened material
/// is the tuple of the materials it samples over the layers the effective
/// palette reads through.
pub(crate) struct Swatches<'a> {
    effective: VoxEffectivePalette<'a>,

    /// The layers supplying at least one property, in the effective
    /// palette's order.
    identity_layer_ids: Vec<U32Id<BVoxLayer>>,

    /// Each swatch's materials over `identity_layer_ids`.
    keys: IdVec<BSwatch, Vec<U32Id<BVoxMaterial>>>,

    /// Each voxel entry's swatch, the entries in raster order.
    voxel_swatch_ids: IdVec<BVoxel, U32Id<BSwatch>>,

    /// Each live voxel's entry.
    voxel_entry_ids: HashMap<U32Id<BVoxVoxel>, U32Id<BVoxel>>,
}

impl<'a> Swatches<'a> {
    /// Resolves `object`'s swatches over `main`'s palettes. Errors if a layer
    /// references a palette `main` does not hold.
    pub(crate) fn resolve<T: VoxExt>(main: &'a VoxMain<T>, object: &'a VoxObject) -> Result<Self> {
        let effective = main.effective_palette(object)?;

        let mut identity_layer_ids = Vec::new();

        for index in 0..effective.property_count() {
            let property = effective
                .property(UsizeId::from_usize(index))
                .expect("property ids below the count resolve");

            if !identity_layer_ids.contains(&property.layer_id()) {
                identity_layer_ids.push(property.layer_id());
            }
        }

        let mut keys = IdVec::default();
        let mut swatch_by_key: HashMap<Vec<U32Id<BVoxMaterial>>, U32Id<BSwatch>> = HashMap::new();
        let mut voxel_swatch_ids = IdVec::default();
        let mut voxel_entry_ids = HashMap::new();

        for voxel_id in object.iter_live() {
            let key: Vec<U32Id<BVoxMaterial>> = identity_layer_ids
                .iter()
                .map(|&layer_id| {
                    object
                        .voxel_material(voxel_id, layer_id)
                        .expect("a live voxel samples a material in each of the object's layers")
                })
                .collect();

            let swatch_id = match swatch_by_key.get(&key) {
                Some(&swatch_id) => swatch_id,

                None => {
                    let swatch_id = U32Id::from_u32(keys.len() as u32);
                    keys.push(key.clone());
                    swatch_by_key.insert(key, swatch_id);
                    swatch_id
                }
            };

            let entry_id = U32Id::from_u32(voxel_swatch_ids.len() as u32);
            voxel_swatch_ids.push(swatch_id);
            voxel_entry_ids.insert(voxel_id, entry_id);
        }

        Ok(Swatches {
            effective,
            identity_layer_ids,
            keys,
            voxel_swatch_ids,
            voxel_entry_ids,
        })
    }

    /// The swatch count.
    pub(crate) fn count(&self) -> usize {
        self.keys.len()
    }

    /// The effective palette the swatches read through.
    pub(crate) fn effective(&self) -> &VoxEffectivePalette<'a> {
        &self.effective
    }

    /// The value swatch `swatch_id` holds for `property_id`.
    pub(crate) fn value(
        &self,
        swatch_id: U32Id<BSwatch>,
        property_id: UsizeId<BVoxEffectiveProperty>,
    ) -> VoxValuePoolValueRef<'_> {
        let property = self
            .effective
            .property(property_id)
            .expect("the property is one of the effective palette's");

        let position = self
            .identity_layer_ids
            .iter()
            .position(|&layer_id| layer_id == property.layer_id())
            .expect("a supplying layer is one of the identity layers");

        let material_id = self.keys[swatch_id.to_usize_id()][position];

        property
            .value(material_id)
            .expect("a material holds a value for every property of its palette")
    }

    /// The entry of the live voxel `voxel_id`.
    pub(crate) fn voxel_entry_id(&self, voxel_id: U32Id<BVoxVoxel>) -> U32Id<BVoxel> {
        *self
            .voxel_entry_ids
            .get(&voxel_id)
            .expect("the voxel is one of the object's live voxels")
    }

    /// Each voxel entry's swatch, the entries in raster order.
    pub(crate) fn voxel_swatch_ids(&self) -> &IdVec<BVoxel, U32Id<BSwatch>> {
        &self.voxel_swatch_ids
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::mesh::Swatches;
    use branded_id::{U32Id, UsizeId};
    use ty_math::TyVector3U32;
    use voxcore::{
        BVoxValuePoolValue, VoxMain, VoxObject, VoxPalette, VoxValuePool, VoxValuePoolValueRef,
        material::{BASE_COLOR, METALLIC},
    };

    fn value_id(index: u32) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index)
    }

    /// A main holding two palettes: `base` carries `baseColor` and
    /// `metallic`, and `over` carries `metallic` alone, each with two
    /// materials.
    fn main() -> (VoxMain, [U32Id<voxcore::BVoxPalette>; 2]) {
        let mut main: VoxMain = VoxMain::default();

        let colors = main.retain_value_pool(
            VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]]).unwrap(),
        );
        let metals = main.retain_value_pool(VoxValuePool::float(vec![1.0, 0.0]).unwrap());
        let overrides = main.retain_value_pool(VoxValuePool::float(vec![0.25, 0.75]).unwrap());

        let mut base = VoxPalette::default();
        base.retain_property(BASE_COLOR.to_owned(), colors, value_id(0))
            .unwrap();
        base.retain_property(METALLIC.to_owned(), metals, value_id(0))
            .unwrap();
        base.retain_material(vec![value_id(0), value_id(0)])
            .unwrap();
        base.retain_material(vec![value_id(1), value_id(1)])
            .unwrap();

        let mut over = VoxPalette::default();
        over.retain_property(METALLIC.to_owned(), overrides, value_id(0))
            .unwrap();
        over.retain_material(vec![value_id(0)]).unwrap();
        over.retain_material(vec![value_id(1)]).unwrap();

        let base_id = main.retain_palette(base).unwrap();
        let over_id = main.retain_palette(over).unwrap();

        (main, [base_id, over_id])
    }

    /// A 3x1x1 bar over `palette_ids`, its voxels sampling `materials`, one
    /// `[material per layer]` per voxel in raster order.
    fn bar(palette_ids: &[U32Id<voxcore::BVoxPalette>], materials: &[&[u32]]) -> VoxObject {
        let mut object = VoxObject::new("bar".to_owned(), TyVector3U32::new(3, 1, 1)).unwrap();

        for &palette_id in palette_ids {
            object.retain_layer(palette_id, U32Id::from_u32(0));
        }

        for (x, samples) in (0..).zip(materials) {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            let samples: Vec<_> = samples.iter().map(|&m| U32Id::from_u32(m)).collect();
            object.retain_voxel(voxel_id, &samples).unwrap();
        }

        object
    }

    #[test]
    fn voxels_sharing_a_material_share_a_swatch_in_first_seen_order() {
        let (main, [base_id, _]) = main();
        let object = bar(&[base_id], &[&[1], &[0], &[1]]);

        let swatches = Swatches::resolve(&main, &object).unwrap();

        assert_eq!(swatches.count(), 2);
        let entries: Vec<u32> = swatches
            .voxel_swatch_ids()
            .iter()
            .map(|swatch_id| swatch_id.to_u32())
            .collect();
        assert_eq!(entries, [0, 1, 0]);
        assert_eq!(
            swatches
                .voxel_entry_id(object.voxel_id(TyVector3U32::new(2, 0, 0)).unwrap())
                .to_u32(),
            2
        );

        let metallic = swatches.effective().property_id_by_name(METALLIC).unwrap();
        assert_eq!(
            swatches.value(U32Id::from_u32(0), metallic),
            VoxValuePoolValueRef::Float(0.0)
        );
        assert_eq!(
            swatches.value(U32Id::from_u32(1), metallic),
            VoxValuePoolValueRef::Float(1.0)
        );
    }

    #[test]
    fn a_later_layer_supplies_its_property_and_splits_the_swatches() {
        let (main, palette_ids) = main();
        // The first two voxels share a base color but not the override layer's
        // metallic, so they are two swatches.
        let object = bar(&palette_ids, &[&[0, 0], &[0, 1], &[1, 1]]);

        let swatches = Swatches::resolve(&main, &object).unwrap();

        assert_eq!(swatches.count(), 3);
        let metallic = swatches.effective().property_id_by_name(METALLIC).unwrap();
        assert_eq!(
            swatches.value(U32Id::from_u32(1), metallic),
            VoxValuePoolValueRef::Float(0.75)
        );
        let color = swatches
            .effective()
            .property_id_by_name(BASE_COLOR)
            .unwrap();
        assert_eq!(
            swatches.value(U32Id::from_u32(2), color),
            VoxValuePoolValueRef::Vec4Float(&[0.0, 0.0, 1.0, 1.0])
        );
        assert_eq!(swatches.effective().property_count(), 2);
        assert!(
            swatches
                .effective()
                .property(UsizeId::from_usize(0))
                .is_some()
        );
    }

    #[test]
    fn an_object_without_layers_has_one_swatch_per_nothing() {
        let (main, _) = main();
        let object = bar(&[], &[&[], &[]]);

        let swatches = Swatches::resolve(&main, &object).unwrap();

        assert_eq!(swatches.count(), 1);
        assert_eq!(swatches.effective().property_count(), 0);
    }
}
