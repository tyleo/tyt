use crate::operations::sdf_doc::WHOLE_TOLERANCE;
use branded_id::U32Id;
use sdfcore::BSdfMaterial;
use ty_math::{TyVector3F64, TyVector3I32, TyVector3U32};

/// One object a document holds, with the material of each live cell.
pub struct ObjectCells {
    /// The part's name.
    pub name: String,

    /// The lattice index of the least cell of the box around the live cells.
    pub min: TyVector3I32,

    /// The cells along each axis of the box around the live cells.
    pub size: TyVector3U32,

    /// Each live cell's position in the box beside its material, in a raster
    /// with x outermost.
    pub cells: Vec<(TyVector3U32, U32Id<BSdfMaterial>)>,

    /// The whole voxels the object's origin adds to `min`. The object's first
    /// place fixes them.
    pub base: TyVector3I32,
}

impl ObjectCells {
    /// The object named `name` holding `cells`, or `None` for no cells. A
    /// later cell at one lattice index wins it. The object's first place sits
    /// `lattice_offset` voxels from the lattice.
    pub fn new(
        name: String,
        cells: &[(TyVector3I32, U32Id<BSdfMaterial>)],
        lattice_offset: TyVector3F64,
    ) -> Option<Self> {
        let (min, max) = cells.iter().fold(None, |bounds, (cell, _)| {
            Some(bounds.map_or(
                (*cell, *cell),
                |(min, max): (TyVector3I32, TyVector3I32)| (min.min(*cell), max.max(*cell)),
            ))
        })?;

        // Reversed, the stable sort puts the last cell at each position first,
        // where the dedup keeps it.
        let mut placed: Vec<(TyVector3U32, U32Id<BSdfMaterial>)> = cells
            .iter()
            .rev()
            .map(|(cell, material_id)| ((*cell - min).as_uvec3(), *material_id))
            .collect();
        placed.sort_by_key(|(position, _)| position.to_array());
        placed.dedup_by_key(|(position, _)| *position);

        let base = TyVector3F64::from_array(lattice_offset.to_array().map(|component| {
            let rounded = component.round();

            if (component - rounded).abs() <= WHOLE_TOLERANCE {
                rounded
            } else {
                component.floor()
            }
        }))
        .as_ivec3();

        Some(Self {
            name,
            min,
            size: (max - min + 1).as_uvec3(),
            cells: placed,
            base,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::ObjectCells;
    use branded_id::U32Id;
    use ty_math::{TyVector3F64, TyVector3I32, TyVector3U32};

    #[test]
    fn a_later_cell_at_one_lattice_index_wins_it() {
        let (first, later) = (U32Id::from_u32(0), U32Id::from_u32(1));
        let object = ObjectCells::new(
            "part".to_string(),
            &[
                (TyVector3I32::new(2, 0, 0), first),
                (TyVector3I32::new(1, 0, 0), first),
                (TyVector3I32::new(2, 0, 0), later),
            ],
            TyVector3F64::ZERO,
        )
        .unwrap();

        assert_eq!(object.min, TyVector3I32::new(1, 0, 0));
        assert_eq!(object.size, TyVector3U32::new(2, 1, 1));
        assert_eq!(
            object.cells,
            vec![
                (TyVector3U32::new(0, 0, 0), first),
                (TyVector3U32::new(1, 0, 0), later),
            ]
        );
    }
}
