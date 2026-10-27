use ty_math::TyVector3F64;

/// An axis-aligned box that holds every point where a shape's distance reads
/// zero or less.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds3d {
    /// The corner with the least coordinates.
    pub min: TyVector3F64,

    /// The corner with the greatest coordinates.
    pub max: TyVector3F64,
}

impl Bounds3d {
    /// The box spanning `center` plus and minus `half_extents`.
    pub fn around(center: TyVector3F64, half_extents: TyVector3F64) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// The box around `points`.
    ///
    /// # Panics
    ///
    /// When `points` is empty.
    pub fn from_points(points: impl IntoIterator<Item = TyVector3F64>) -> Self {
        points
            .into_iter()
            .map(|point| Self {
                min: point,
                max: point,
            })
            .reduce(|bounds, point| bounds.union(&point))
            .expect("a box wraps at least one point")
    }

    /// The box's eight corners.
    pub fn corners(&self) -> [TyVector3F64; 8] {
        let Self { min, max } = *self;

        [
            TyVector3F64::new(min.x, min.y, min.z),
            TyVector3F64::new(max.x, min.y, min.z),
            TyVector3F64::new(min.x, max.y, min.z),
            TyVector3F64::new(max.x, max.y, min.z),
            TyVector3F64::new(min.x, min.y, max.z),
            TyVector3F64::new(max.x, min.y, max.z),
            TyVector3F64::new(min.x, max.y, max.z),
            TyVector3F64::new(max.x, max.y, max.z),
        ]
    }

    /// The box grown by `by` on each side.
    pub fn grow(&self, by: TyVector3F64) -> Self {
        Self {
            min: self.min - by,
            max: self.max + by,
        }
    }

    /// The overlap of this box and `other`. The overlap of two boxes that never
    /// meet has a min corner past its max corner.
    pub fn intersection(&self, other: &Self) -> Self {
        Self {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        }
    }

    /// The box around this box and `other`.
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}
