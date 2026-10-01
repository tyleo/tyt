use crate::{Dimension, Domain, ValueEnvironment, f64s, groupings};
use std::collections::HashMap;

/// The step: one stone swatch, three voxels in an L, and ten greedy faces,
/// the bottom and the left side merged across two voxels each and the front
/// and back split into a tall quad and a short one.
pub fn step() -> ValueEnvironment {
    let mut environment = ValueEnvironment {
        values: HashMap::new(),
        groupings: groupings(
            1,
            &[0, 0, 0],
            &[
                &[0, 1],
                &[0, 2],
                &[0, 2],
                &[1],
                &[0, 2],
                &[1],
                &[2],
                &[1],
                &[1],
                &[2],
            ],
        ),
    };

    environment.values = [
        (
            "baseColor",
            f64s(Domain::Swatch, Dimension::Vec4, &[0.55, 0.5, 0.45, 1.0]),
        ),
        (
            "height",
            f64s(Domain::Voxel, Dimension::Vec1, &[0.0, 0.0, 1.0]),
        ),
        (
            "faceValue",
            f64s(
                Domain::Face,
                Dimension::Vec1,
                &(0..10).map(|face| face as f64).collect::<Vec<_>>(),
            ),
        ),
        (
            "computedOcclusion",
            f64s(
                Domain::Corner,
                Dimension::Vec1,
                &(0..40).map(|corner| corner as f64).collect::<Vec<_>>(),
            ),
        ),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value))
    .collect();

    environment
}
