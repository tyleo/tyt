use crate::{
    BVoxEffectiveProperty, BVoxMaterial, BVoxVoxel, VoxEffectiveProperty, VoxObject,
    VoxValuePoolValueRef,
};
use branded_id::{IdVec, U32Id, UsizeId};
use std::collections::HashMap;

/// A per-object view resolving the layer override rule once: each property
/// reads through the last layer whose palette supplies its name.
///
/// Build with
/// [`VoxMain::effective_palette`](crate::VoxMain::effective_palette). Resolve
/// a name to an id once with
/// [`property_id_by_name`](Self::property_id_by_name), then read values by id
/// inside voxel loops.
#[derive(Debug)]
pub struct VoxEffectivePalette<'a> {
    /// The object the view resolves.
    pub(crate) object: &'a VoxObject,

    /// The resolved properties. An override keeps its first supplier's id,
    /// so the listing is stable.
    pub(crate) properties: IdVec<BVoxEffectiveProperty, VoxEffectiveProperty<'a>>,

    /// Name index into `properties`.
    pub(crate) property_id_by_name: HashMap<&'a str, UsizeId<BVoxEffectiveProperty>>,
}

impl<'a> VoxEffectivePalette<'a> {
    /// The property `id`, or `None` if not one of this palette's.
    pub fn property(
        &self,
        id: UsizeId<BVoxEffectiveProperty>,
    ) -> Option<&VoxEffectiveProperty<'a>> {
        self.properties.get(id)
    }

    /// Number of properties.
    pub fn property_count(&self) -> usize {
        self.properties.len()
    }

    /// The property named `name`, or `None` when no layer supplies it. O(1)
    /// through the name index.
    pub fn property_id_by_name(&self, name: &str) -> Option<UsizeId<BVoxEffectiveProperty>> {
        self.property_id_by_name.get(name).copied()
    }

    /// The value `material_id` draws for property `property_id`, or `None`
    /// if either id is unknown.
    pub fn value(
        &self,
        property_id: UsizeId<BVoxEffectiveProperty>,
        material_id: U32Id<BVoxMaterial>,
    ) -> Option<VoxValuePoolValueRef<'_>> {
        self.properties.get(property_id)?.value(material_id)
    }

    /// The value the voxel at `voxel_id` reads for property `property_id`,
    /// through the material it samples in the winning layer. `None` for a
    /// dead voxel or an unknown property id.
    pub fn voxel_value(
        &self,
        voxel_id: U32Id<BVoxVoxel>,
        property_id: UsizeId<BVoxEffectiveProperty>,
    ) -> Option<VoxValuePoolValueRef<'_>> {
        let property = self.properties.get(property_id)?;
        let material_id = self.object.voxel_material(voxel_id, property.layer_id)?;
        property.value(material_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BVoxPalette, BVoxValuePool, BVoxValuePoolValue, Error, VoxMain, VoxObject, VoxPalette,
        VoxValuePool, VoxValuePoolValueRef,
    };
    use branded_id::{U32Id, UsizeId};
    use ty_math::TyVector3U32;

    fn value_id(index: u32) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index)
    }

    /// Retains an `int` value pool holding `values` and returns its id.
    fn int_value_pool(state: &mut VoxMain, values: Vec<i64>) -> U32Id<BVoxValuePool> {
        state.retain_value_pool(VoxValuePool::int(values).unwrap())
    }

    /// Retains a palette with one property per `(name, value id)` entry on
    /// `value_pool_id` and one material drawing the listed value ids, and
    /// returns its id.
    fn palette_over(
        state: &mut VoxMain,
        value_pool_id: U32Id<BVoxValuePool>,
        entries: &[(&str, u32)],
    ) -> U32Id<BVoxPalette> {
        let mut palette = VoxPalette::default();
        for &(name, _) in entries {
            palette
                .retain_property(name.to_owned(), value_pool_id, value_id(0))
                .unwrap();
        }
        palette
            .retain_material(entries.iter().map(|&(_, index)| value_id(index)).collect())
            .unwrap();
        state.retain_palette(palette).unwrap()
    }

    /// A one-voxel object layering the given palettes in order, its voxel
    /// live and sampling material 0 in every layer.
    fn object_over(palette_ids: &[U32Id<BVoxPalette>]) -> VoxObject {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        for &palette_id in palette_ids {
            object.retain_layer(palette_id, U32Id::from_u32(0));
        }
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        let sample_ids = vec![U32Id::from_u32(0); palette_ids.len()];
        object.retain_voxel(voxel_id, &sample_ids).unwrap();
        object
    }

    #[test]
    fn the_last_supplying_layer_wins() {
        let mut state = VoxMain::default();
        let value_pool_id = int_value_pool(&mut state, vec![10, 20]);
        let first_id = palette_over(&mut state, value_pool_id, &[("v", 0)]);
        let second_id = palette_over(&mut state, value_pool_id, &[("v", 1)]);
        let object = object_over(&[first_id, second_id]);

        let effective = state.effective_palette(&object).unwrap();
        assert_eq!(effective.property_count(), 1);
        let property_id = effective.property_id_by_name("v").unwrap();
        let property = effective.property(property_id).unwrap();
        let layers: Vec<_> = object.iter_layers().collect();
        assert_eq!(property.name(), "v");
        assert_eq!(property.layer_id(), layers[1].0);
        assert_eq!(property.palette_id(), second_id);

        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        assert_eq!(
            effective.voxel_value(voxel_id, property_id),
            Some(VoxValuePoolValueRef::Int(20))
        );
    }

    #[test]
    fn a_dead_voxel_reads_no_value() {
        let mut state = VoxMain::default();
        let value_pool_id = int_value_pool(&mut state, vec![10]);
        let palette_id = palette_over(&mut state, value_pool_id, &[("v", 0)]);
        let mut object = object_over(&[palette_id]);
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.release_voxel(voxel_id).unwrap();

        let effective = state.effective_palette(&object).unwrap();
        let property_id = effective.property_id_by_name("v").unwrap();
        assert_eq!(effective.voxel_value(voxel_id, property_id), None);
    }

    #[test]
    fn a_layer_whose_palette_lacks_the_name_is_passed_over() {
        let mut state = VoxMain::default();
        let value_pool_id = int_value_pool(&mut state, vec![10, 20]);
        let supplying_id = palette_over(&mut state, value_pool_id, &[("v", 0)]);
        let plain_id = palette_over(&mut state, value_pool_id, &[("w", 1)]);
        let object = object_over(&[supplying_id, plain_id]);

        let effective = state.effective_palette(&object).unwrap();
        let layers: Vec<_> = object.iter_layers().collect();
        let v = effective
            .property(effective.property_id_by_name("v").unwrap())
            .unwrap();
        assert_eq!((v.layer_id(), v.palette_id()), (layers[0].0, supplying_id));
        let w = effective
            .property(effective.property_id_by_name("w").unwrap())
            .unwrap();
        assert_eq!((w.layer_id(), w.palette_id()), (layers[1].0, plain_id));
    }

    #[test]
    fn an_override_keeps_the_first_seen_id() {
        let mut state = VoxMain::default();
        let value_pool_id = int_value_pool(&mut state, vec![10, 20, 30]);
        let first_id = palette_over(&mut state, value_pool_id, &[("a", 0), ("b", 1)]);
        let second_id = palette_over(&mut state, value_pool_id, &[("a", 2)]);
        let object = object_over(&[first_id, second_id]);

        let effective = state.effective_palette(&object).unwrap();
        let a_id = effective.property_id_by_name("a").unwrap();
        let b_id = effective.property_id_by_name("b").unwrap();
        assert_eq!(a_id, UsizeId::from_usize(0));
        assert_eq!(b_id, UsizeId::from_usize(1));
        assert_eq!(effective.property(a_id).unwrap().palette_id(), second_id);
        assert_eq!(effective.property(b_id).unwrap().palette_id(), first_id);
    }

    #[test]
    fn a_sparse_material_id_reads_correctly_before_gc() {
        let mut state = VoxMain::default();
        let value_pool_id = int_value_pool(&mut state, vec![10, 20, 30]);
        let mut palette = VoxPalette::default();
        palette
            .retain_property("v".to_owned(), value_pool_id, value_id(0))
            .unwrap();
        let keep_id = palette.retain_material(vec![value_id(0)]).unwrap();
        let doomed_id = palette.retain_material(vec![value_id(1)]).unwrap();
        let sparse_id = palette.retain_material(vec![value_id(2)]).unwrap();
        let palette_id = state.retain_palette(palette).unwrap();
        state
            .release_material(palette_id, doomed_id, keep_id)
            .unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.retain_layer(palette_id, sparse_id);
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel_id, &[sparse_id]).unwrap();

        let effective = state.effective_palette(&object).unwrap();
        let v_id = effective.property_id_by_name("v").unwrap();
        assert_eq!(
            effective.value(v_id, sparse_id),
            Some(VoxValuePoolValueRef::Int(30))
        );
        assert_eq!(
            effective.value(v_id, keep_id),
            Some(VoxValuePoolValueRef::Int(10))
        );
        assert_eq!(effective.value(v_id, doomed_id), None);
        assert_eq!(
            effective.voxel_value(voxel_id, v_id),
            Some(VoxValuePoolValueRef::Int(30))
        );
    }

    #[test]
    fn an_object_with_no_layers_yields_an_empty_palette() {
        let state = VoxMain::default();
        let object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();

        let effective = state.effective_palette(&object).unwrap();
        assert_eq!(effective.property_count(), 0);
        assert_eq!(effective.property_id_by_name("v"), None);
        assert_eq!(
            effective.value(UsizeId::from_usize(0), U32Id::from_u32(0)),
            None
        );
    }

    #[test]
    fn a_layer_over_an_unknown_palette_errors() {
        let state = VoxMain::default();
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        let layer_id = object.retain_layer(U32Id::from_u32(9), U32Id::from_u32(0));

        assert_eq!(
            state.effective_palette(&object).unwrap_err(),
            Error::LayerPaletteRef {
                layer_id,
                palette_id: U32Id::from_u32(9),
            }
        );
    }
}
