use crate::{Domain, Groupings, evaluator::Lengths};
use branded_id::UsizeId;

/// The source entries each destination entry reduces, a merged face
/// counting once per piece.
pub(crate) fn groups(
    source: Domain,
    target: Domain,
    groupings: &Groupings,
    lengths: &Lengths,
) -> Vec<Vec<usize>> {
    let mut groups = vec![Vec::new(); lengths.of(target)];
    let corners_of = |face: usize| face * 4..(face + 1) * 4;
    let swatch_of = |voxel: usize| {
        groupings.voxel_swatches[UsizeId::from_usize(voxel)]
            .to_usize_id()
            .to_usize()
    };

    match target {
        Domain::Plain => groups[0].extend(0..lengths.of(source)),

        Domain::Swatch if source == Domain::Voxel => {
            for voxel in 0..lengths.voxels {
                groups[swatch_of(voxel)].push(voxel);
            }
        }

        Domain::Swatch | Domain::Voxel => {
            for (face, pieces) in groupings.face_voxels.iter().enumerate() {
                for voxel_id in pieces {
                    let voxel = voxel_id.to_usize_id().to_usize();
                    let destination = if target == Domain::Voxel {
                        voxel
                    } else {
                        swatch_of(voxel)
                    };

                    match source {
                        Domain::Face => groups[destination].push(face),
                        Domain::Corner => groups[destination].extend(corners_of(face)),
                        _ => {
                            unreachable!("the checker keeps a reduction's source above its target")
                        }
                    }
                }
            }
        }

        Domain::Face => {
            for (face, group) in groups.iter_mut().enumerate() {
                group.extend(corners_of(face));
            }
        }

        Domain::Corner => unreachable!("nothing reduces onto corner"),
    }

    groups
}
