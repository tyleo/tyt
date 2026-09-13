use crate::Result;
use branded_id::{U32Id, UsizeId};
use std::collections::HashMap;
use voxcore::{
    BVoxEffectiveProperty, BVoxLayer, BVoxMaterial, BVoxValuePoolValue, VoxEffectivePalette,
    VoxEffectiveProperty, VoxExt, VoxMain, VoxObject,
    material::{BASE_COLOR, EMISSIVE_COLOR},
};

/// An object's effective palette with its properties split by the Voxel Max
/// axis each writes to, and its layers in order for reading a voxel's samples.
pub(crate) struct PaletteAxes<'a> {
    pub(crate) effective: VoxEffectivePalette<'a>,

    /// Each layer's position in a voxel's sample tuple.
    pub(crate) layer_positions: HashMap<U32Id<BVoxLayer>, usize>,

    /// `baseColor`, one value per color cell.
    pub(crate) color: Option<UsizeId<BVoxEffectiveProperty>>,

    /// `emissiveColor`, one value per color cell.
    pub(crate) emissive_color: Option<UsizeId<BVoxEffectiveProperty>>,

    /// Every other property, one value per material slot, in effective order.
    pub(crate) material: Vec<UsizeId<BVoxEffectiveProperty>>,
}

impl<'a> PaletteAxes<'a> {
    /// Resolves `object`'s layers once. Errors when a layer references a
    /// palette the state does not hold.
    pub(crate) fn resolve<T: VoxExt>(main: &'a VoxMain<T>, object: &'a VoxObject) -> Result<Self> {
        let effective = main.effective_palette(object)?;
        let layer_positions = object
            .iter_layers()
            .enumerate()
            .map(|(position, (layer_id, _))| (layer_id, position))
            .collect();
        let color = effective.property_id_by_name(BASE_COLOR);
        let emissive_color = effective.property_id_by_name(EMISSIVE_COLOR);
        let material = (0..effective.property_count())
            .map(UsizeId::from_usize)
            .filter(|&property_id| {
                Some(property_id) != color && Some(property_id) != emissive_color
            })
            .collect();
        Ok(PaletteAxes {
            effective,
            layer_positions,
            color,
            emissive_color,
            material,
        })
    }

    /// The property `property_id`.
    pub(crate) fn property(
        &self,
        property_id: UsizeId<BVoxEffectiveProperty>,
    ) -> &VoxEffectiveProperty<'a> {
        self.effective
            .property(property_id)
            .expect("an axis holds the effective palette's own property ids")
    }

    /// The value id a voxel sampling `sample` draws for `property_id`, read
    /// through the material it samples in the property's layer.
    pub(crate) fn value_id(
        &self,
        property_id: UsizeId<BVoxEffectiveProperty>,
        sample: &[U32Id<BVoxMaterial>],
    ) -> U32Id<BVoxValuePoolValue> {
        let property = self.property(property_id);
        let material_id = sample[self.layer_positions[&property.layer_id()]];
        property
            .value_id(material_id)
            .expect("a live voxel samples a material of its layer's palette")
    }

    /// The position of the material-axis property named `name`, or `None` when
    /// no layer supplies it.
    pub(crate) fn material_position(&self, name: &str) -> Option<usize> {
        self.material
            .iter()
            .position(|&property_id| self.property(property_id).name() == name)
    }
}
