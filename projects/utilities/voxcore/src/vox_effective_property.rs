use crate::{
    BVoxLayer, BVoxMaterial, BVoxPalette, BVoxProperty, BVoxValuePoolValue, VoxPalette,
    VoxValuePool, VoxValuePoolValueRef,
};
use branded_id::U32Id;

/// One resolved property of a
/// [`VoxEffectivePalette`](crate::VoxEffectivePalette): the winning supplier,
/// read by the material a voxel samples in the winning layer.
#[derive(Debug)]
pub struct VoxEffectiveProperty<'a> {
    /// The property name.
    pub(crate) name: &'a str,

    /// The winning layer.
    pub(crate) layer_id: U32Id<BVoxLayer>,

    /// The palette the winning layer references.
    pub(crate) palette_id: U32Id<BVoxPalette>,

    /// That palette.
    pub(crate) palette: &'a VoxPalette,

    /// The supplying property in the winning palette.
    pub(crate) property_id: U32Id<BVoxProperty>,

    /// The value pool the property draws values from.
    pub(crate) value_pool: &'a VoxValuePool,
}

impl<'a> VoxEffectiveProperty<'a> {
    /// The winning layer.
    pub fn layer_id(&self) -> U32Id<BVoxLayer> {
        self.layer_id
    }

    /// The property name.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// The palette the winning layer references.
    pub fn palette_id(&self) -> U32Id<BVoxPalette> {
        self.palette_id
    }

    /// The winning palette's materials with the value id each draws, in
    /// material order.
    pub fn iter_values(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxMaterial>, U32Id<BVoxValuePoolValue>)> + '_ {
        self.palette.iter_materials().map(move |material_id| {
            let value_id = self
                .palette
                .value_id(material_id, self.property_id)
                .expect("a material has a value id for every property");
            (material_id, value_id)
        })
    }

    /// The value `material_id` draws, or `None` if `material_id` is not one of
    /// the winning palette's materials.
    pub fn value(&self, material_id: U32Id<BVoxMaterial>) -> Option<VoxValuePoolValueRef<'_>> {
        let value_id = self.value_id(material_id)?;
        self.value_pool.value(value_id)
    }

    /// The value id `material_id` draws, or `None` if `material_id` is not one
    /// of the winning palette's materials.
    pub fn value_id(&self, material_id: U32Id<BVoxMaterial>) -> Option<U32Id<BVoxValuePoolValue>> {
        self.palette.value_id(material_id, self.property_id)
    }

    /// The value pool the property draws values from.
    pub fn value_pool(&self) -> &'a VoxValuePool {
        self.value_pool
    }
}
