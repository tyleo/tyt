use crate::{CellColor, Result, resolve_cell_color};
use voxcore::{VoxMain, VoxObject};

/// [`resolve_cell_color`], reading transparent black when no layer supplies
/// `baseColorFactor`.
pub fn resolve_cell_color_or_transparent<'a>(
    state: &VoxMain,
    object: &'a VoxObject,
) -> Result<CellColor<'a>> {
    Ok(resolve_cell_color(state, object)?.unwrap_or_else(|| Box::new(|_| [0, 0, 0, 0])))
}

#[cfg(test)]
mod tests {
    use crate::resolve_cell_color_or_transparent;
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{VoxMain, VoxObject, VoxPalette};

    #[test]
    fn an_unsupplied_object_reads_transparent_black() {
        let mut state = VoxMain::default();
        // A property-less palette: its one material supplies no
        // baseColorFactor.
        let mut palette = VoxPalette::default();
        palette.add_material(vec![]).unwrap();
        let palette_id = state.add_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.add_layer(palette_id, U32Id::from_u32(0));
        let voxel = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel, &[U32Id::from_u32(0)]).unwrap();

        let cell_color = resolve_cell_color_or_transparent(&state, &object).unwrap();
        assert_eq!(cell_color(voxel), [0, 0, 0, 0]);
    }
}
