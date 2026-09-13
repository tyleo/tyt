use crate::{Error, LayoutProperty, Result};
use branded_id::U32Id;
use voxcore::{
    BVoxPalette, VoxExt, VoxMain, VoxPalette,
    material::{BASE_COLOR, EMISSIVE_COLOR},
};

/// A palette read in Voxel Max's layout, the one
/// [`from_vmax_file`](crate::from_vmax_file) builds. The color axis is
/// `baseColor` and `emissiveColor`, one value per color cell. The material
/// axis is every other property, one value per material slot. A material is
/// one cell with one slot, so its color-axis properties share a value id and
/// its material-axis properties share another. The writer converts nothing:
/// a palette off the layout errors where it departs.
pub(crate) struct PaletteLayout<'a> {
    pub(crate) palette: &'a VoxPalette,

    /// `baseColor`.
    pub(crate) color: Option<LayoutProperty<'a>>,

    /// `emissiveColor`.
    pub(crate) emissive_color: Option<LayoutProperty<'a>>,

    /// The material axis, in property order.
    pub(crate) material: Vec<LayoutProperty<'a>>,
}

impl<'a> PaletteLayout<'a> {
    /// Reads palette `palette_id` of `main`. Errors when the state does not
    /// hold it.
    pub(crate) fn resolve<T: VoxExt>(
        main: &'a VoxMain<T>,
        palette_id: U32Id<BVoxPalette>,
    ) -> Result<Self> {
        let palette = main.palette(palette_id).ok_or_else(|| {
            Error::invalid(format!(
                "an object layers palette {}, which the state does not hold",
                palette_id.to_u32()
            ))
        })?;
        let mut color = None;
        let mut emissive_color = None;
        let mut material = Vec::new();
        for (id, property) in palette.iter_properties() {
            let value_pool = main
                .value_pool(property.value_pool_id)
                .expect("a property draws from a live value pool");
            let entry = LayoutProperty {
                id,
                name: &property.name,
                value_pool,
            };
            match property.name.as_str() {
                BASE_COLOR => color = Some(entry),
                EMISSIVE_COLOR => emissive_color = Some(entry),
                _ => material.push(entry),
            }
        }
        Ok(PaletteLayout {
            palette,
            color,
            emissive_color,
            material,
        })
    }

    /// The material-axis property named `name`, or `None` when the palette
    /// does not bind it.
    pub(crate) fn material_property(&self, name: &str) -> Option<&LayoutProperty<'a>> {
        self.material.iter().find(|property| property.name == name)
    }
}
