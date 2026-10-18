use crate::operations::sdf_doc::{FACE_OFFSETS, SdfGrid};
use branded_id::IdSlice;
use sdfcore::BSdfMaterial;
use ty_math::TyVector3I32;

/// Whether the live `cell` of `grid` shows: it meets an empty cell across a
/// face, or it holds an opaque material and meets a cell that lets light
/// through. A cell outside the grid counts as empty.
pub fn is_exposed(
    grid: &SdfGrid,
    cell: TyVector3I32,
    see_through: &IdSlice<BSdfMaterial, bool>,
) -> bool {
    let material_id = grid
        .cell(cell)
        .and_then(|cell| cell.material)
        .expect("an exposed cell is live");

    FACE_OFFSETS.iter().any(|face| {
        match grid
            .cell(cell + *face)
            .and_then(|neighbor| neighbor.material)
        {
            Some(neighbor_id) => {
                !see_through[material_id.to_usize_id()] && see_through[neighbor_id.to_usize_id()]
            }

            None => true,
        }
    })
}
