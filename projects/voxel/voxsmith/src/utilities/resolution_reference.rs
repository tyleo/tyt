use ty_math::TyVector3F64;

/// The side a resolution count divides. World references measure every
/// placed object's bounds together, after every node transform. Object
/// references measure each object's bounds and take the extreme across
/// objects. A side with no extent never counts as shortest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolutionReference {
    /// The longest side of the world bounds.
    LongestWorld,

    /// The shortest side of the world bounds.
    ShortestWorld,

    /// The world bounds' extent along x.
    WorldX,

    /// The world bounds' extent along y.
    WorldY,

    /// The world bounds' extent along z.
    WorldZ,

    /// The longest side of any object's bounds.
    LongestObject,

    /// The shortest side of any object's bounds.
    ShortestObject,

    /// The longest extent along x of any object's bounds.
    LongestObjectX,

    /// The longest extent along y of any object's bounds.
    LongestObjectY,

    /// The longest extent along z of any object's bounds.
    LongestObjectZ,

    /// The shortest extent along x of any object's bounds.
    ShortestObjectX,

    /// The shortest extent along y of any object's bounds.
    ShortestObjectY,

    /// The shortest extent along z of any object's bounds.
    ShortestObjectZ,
}

impl ResolutionReference {
    /// Whether this is a world reference.
    pub fn is_world(self) -> bool {
        matches!(
            self,
            Self::LongestWorld | Self::ShortestWorld | Self::WorldX | Self::WorldY | Self::WorldZ
        )
    }

    /// The side this reference measures over the `world` extent and each
    /// object's extent in `objects`, or zero when it has none.
    pub fn side(self, world: TyVector3F64, objects: &[TyVector3F64]) -> f64 {
        let axis = |extent: TyVector3F64, axis: usize| extent.to_array()[axis];
        let longest_of = |extent: TyVector3F64| extent.to_array().into_iter().fold(0.0, f64::max);
        let shortest_of = |extent: TyVector3F64| shortest_positive(extent.to_array());
        let longest_object = |side: &dyn Fn(TyVector3F64) -> f64| {
            objects
                .iter()
                .map(|&extent| side(extent))
                .fold(0.0, f64::max)
        };
        let shortest_object = |side: &dyn Fn(TyVector3F64) -> f64| {
            shortest_positive(objects.iter().map(|&extent| side(extent)))
        };

        match self {
            Self::LongestWorld => longest_of(world),
            Self::ShortestWorld => shortest_of(world),
            Self::WorldX => axis(world, 0),
            Self::WorldY => axis(world, 1),
            Self::WorldZ => axis(world, 2),
            Self::LongestObject => longest_object(&longest_of),
            Self::ShortestObject => shortest_object(&shortest_of),
            Self::LongestObjectX => longest_object(&|extent| axis(extent, 0)),
            Self::LongestObjectY => longest_object(&|extent| axis(extent, 1)),
            Self::LongestObjectZ => longest_object(&|extent| axis(extent, 2)),
            Self::ShortestObjectX => shortest_object(&|extent| axis(extent, 0)),
            Self::ShortestObjectY => shortest_object(&|extent| axis(extent, 1)),
            Self::ShortestObjectZ => shortest_object(&|extent| axis(extent, 2)),
        }
    }
}

/// The smallest positive side, or zero when none is positive.
fn shortest_positive(sides: impl IntoIterator<Item = f64>) -> f64 {
    sides
        .into_iter()
        .filter(|&side| side > 0.0)
        .fold(0.0, |shortest, side| {
            if shortest > 0.0 {
                shortest.min(side)
            } else {
                side
            }
        })
}

#[cfg(test)]
mod tests {
    use crate::utilities::ResolutionReference;
    use ty_math::TyVector3F64;

    /// The extent `(x, y, z)`.
    fn extent(x: f64, y: f64, z: f64) -> TyVector3F64 {
        TyVector3F64::new(x, y, z)
    }

    /// Every reference's side over a world of `[3, 2, 0]` holding objects of
    /// `[3, 1, 0]` and `[2, 2, 0]`.
    fn side(reference: ResolutionReference) -> f64 {
        reference.side(
            extent(3.0, 2.0, 0.0),
            &[extent(3.0, 1.0, 0.0), extent(2.0, 2.0, 0.0)],
        )
    }

    #[test]
    fn world_references_measure_the_whole_and_skip_flat_sides_for_shortest() {
        assert_eq!(side(ResolutionReference::LongestWorld), 3.0);
        assert_eq!(side(ResolutionReference::ShortestWorld), 2.0);
        assert_eq!(side(ResolutionReference::WorldX), 3.0);
        assert_eq!(side(ResolutionReference::WorldY), 2.0);
        assert_eq!(side(ResolutionReference::WorldZ), 0.0);
    }

    #[test]
    fn object_references_take_the_extreme_across_objects() {
        assert_eq!(side(ResolutionReference::LongestObject), 3.0);
        assert_eq!(side(ResolutionReference::ShortestObject), 1.0);
        assert_eq!(side(ResolutionReference::LongestObjectX), 3.0);
        assert_eq!(side(ResolutionReference::ShortestObjectX), 2.0);
        assert_eq!(side(ResolutionReference::LongestObjectY), 2.0);
        assert_eq!(side(ResolutionReference::ShortestObjectY), 1.0);
        assert_eq!(side(ResolutionReference::LongestObjectZ), 0.0);
        assert_eq!(side(ResolutionReference::ShortestObjectZ), 0.0);
    }
}
