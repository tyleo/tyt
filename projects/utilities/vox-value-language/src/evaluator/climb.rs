use crate::{
    Domain, EvalFailure, Groupings, Value,
    evaluator::{EntryTransform, EvalResult, Lengths, transform_entries},
};

/// Lifts a value to a domain at or above its own, duplicating entries up the
/// ladder. A value already there comes back untouched.
pub(crate) fn climb(
    value: &Value,
    target: Domain,
    groupings: &Groupings,
    lengths: &Lengths,
) -> EvalResult<Value> {
    if value.domain() == target {
        return Ok(value.clone());
    }

    let transform = Climb {
        from: value.domain(),
        to: target,
        width: value.dimension().width(),
        groupings,
        lengths,
    };
    let components = transform_entries(value.components(), &transform)?;

    Ok(Value::new(target, value.dimension(), components)
        .expect("a climb fills every entry of the target"))
}

struct Climb<'a> {
    from: Domain,
    to: Domain,
    width: usize,
    groupings: &'a Groupings,
    lengths: &'a Lengths,
}

impl EntryTransform for Climb<'_> {
    fn apply<T: Clone + PartialEq>(&self, components: &[T]) -> EvalResult<Vec<T>> {
        let mut current = components.to_vec();
        let mut domain = self.from;

        while domain < self.to {
            let next = match domain {
                Domain::Plain => Domain::Swatch,
                Domain::Swatch => Domain::Voxel,
                Domain::Voxel => Domain::Face,
                Domain::Face => Domain::Corner,
                Domain::Corner => unreachable!("corner is the ladder's top"),
            };

            current = self.step(&current, next)?;
            domain = next;
        }

        Ok(current)
    }
}

impl Climb<'_> {
    /// Lifts entries one rung onto the given domain.
    fn step<T: Clone + PartialEq>(&self, current: &[T], to: Domain) -> EvalResult<Vec<T>> {
        let width = self.width;
        let entry = |index: usize| &current[index * width..(index + 1) * width];
        let mut next = Vec::with_capacity(self.lengths.of(to) * width);

        match to {
            Domain::Swatch => {
                for _ in 0..self.lengths.swatches {
                    next.extend_from_slice(entry(0));
                }
            }

            Domain::Voxel => {
                for swatch_id in self.groupings.voxel_swatches.iter() {
                    next.extend_from_slice(entry(swatch_id.to_usize_id().to_usize()));
                }
            }

            Domain::Face => {
                for (face, pieces) in self.groupings.face_voxels.iter().enumerate() {
                    let first = entry(pieces[0].to_usize_id().to_usize());

                    if pieces
                        .iter()
                        .any(|voxel_id| entry(voxel_id.to_usize_id().to_usize()) != first)
                    {
                        return Err(EvalFailure::ClimbDisagreement {
                            target: Domain::Face,
                            entry: face,
                        });
                    }

                    next.extend_from_slice(first);
                }
            }

            Domain::Corner => {
                for face in 0..self.lengths.faces {
                    for _ in 0..4 {
                        next.extend_from_slice(entry(face));
                    }
                }
            }

            Domain::Plain => unreachable!("nothing climbs onto plain"),
        }

        Ok(next)
    }
}
