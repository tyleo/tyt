use crate::{Domain, Error, Groupings, Result};

/// The entry count of every domain, fixed by the groupings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Lengths {
    pub(crate) swatches: usize,
    pub(crate) voxels: usize,
    pub(crate) faces: usize,
    pub(crate) corners: usize,
}

impl Lengths {
    /// Derives the lengths, erroring on a face with no pieces or a piece
    /// outside the voxel table.
    pub(crate) fn from_groupings(groupings: &Groupings) -> Result<Lengths> {
        let voxels = groupings.voxel_swatches.len();
        let swatches = groupings
            .voxel_swatches
            .iter()
            .map(|swatch_id| swatch_id.to_usize_id().to_usize() + 1)
            .max()
            .unwrap_or(0);

        for (face, pieces) in groupings.face_voxels.iter().enumerate() {
            if pieces.is_empty() {
                return Err(Error::FacePieces { face });
            }

            if let Some(voxel_id) = pieces
                .iter()
                .find(|voxel_id| voxel_id.to_usize_id().to_usize() >= voxels)
            {
                return Err(Error::PieceVoxel {
                    face,
                    voxel: voxel_id.to_u32(),
                });
            }
        }

        let faces = groupings.face_voxels.len();

        Ok(Lengths {
            swatches,
            voxels,
            faces,
            corners: faces * 4,
        })
    }

    /// The entry count of a domain, one for plain.
    pub(crate) fn of(self, domain: Domain) -> usize {
        match domain {
            Domain::Plain => 1,
            Domain::Swatch => self.swatches,
            Domain::Voxel => self.voxels,
            Domain::Face => self.faces,
            Domain::Corner => self.corners,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Domain, Groupings,
        evaluator::{Lengths, groupings},
    };

    #[test]
    fn the_swatch_count_is_one_past_the_largest_swatch_a_voxel_names() {
        let lengths = Lengths::from_groupings(&groupings(&[0, 3, 1], &[&[0, 1], &[2]])).unwrap();

        assert_eq!(
            lengths,
            Lengths {
                swatches: 4,
                voxels: 3,
                faces: 2,
                corners: 8
            }
        );
        assert_eq!(lengths.of(Domain::Plain), 1);
        assert_eq!(lengths.of(Domain::Swatch), 4);
        assert_eq!(lengths.of(Domain::Voxel), 3);
        assert_eq!(lengths.of(Domain::Face), 2);
        assert_eq!(lengths.of(Domain::Corner), 8);
    }

    #[test]
    fn no_voxels_means_no_swatches() {
        assert_eq!(
            Lengths::from_groupings(&Groupings::default()).unwrap(),
            Lengths {
                swatches: 0,
                voxels: 0,
                faces: 0,
                corners: 0
            }
        );
    }
}
