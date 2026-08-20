/// Generates a concrete axis-aligned bounding box over a glam vector and its
/// scalar.
macro_rules! impl_ty_bounds {
    ($name:ident, $vec:ty, $scalar:ty) => {
        /// An axis-aligned bounding box: a center and per-axis half-extents.
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $name {
            /// The center of the box.
            pub center: $vec,

            /// The half-size on each axis.
            pub extents: $vec,
        }

        impl $name {
            /// Creates a box from its `center` and `extents`.
            pub fn new(center: $vec, extents: $vec) -> Self {
                Self { center, extents }
            }

            /// The maximum corner, `center + extents`.
            pub fn max(&self) -> $vec {
                self.center + self.extents
            }

            /// The minimum corner, `center - extents`.
            pub fn min(&self) -> $vec {
                self.center - self.extents
            }

            /// This box scaled about the origin by `value`.
            pub fn scale(&self, value: $scalar) -> Self {
                Self {
                    center: self.center * value,
                    extents: self.extents * value,
                }
            }

            /// The smallest box containing every point in `points`, or `None` when
            /// the iterator is empty.
            pub fn from_points(points: impl IntoIterator<Item = $vec>) -> Option<Self> {
                let mut points = points.into_iter();
                let first = points.next()?;
                let (min, max) = points.fold((first, first), |(min, max), point| {
                    (min.min(point), max.max(point))
                });

                Some(Self {
                    center: (min + max) * 0.5,
                    extents: (max - min) * 0.5,
                })
            }

            /// A box with minimum corner `min` and full `size` on each axis.
            pub fn from_min_size(min: $vec, size: $vec) -> Self {
                let extents = size * 0.5;
                Self {
                    center: min + extents,
                    extents,
                }
            }

            /// The full size on each axis, `extents * 2`.
            pub fn size(&self) -> $vec {
                self.extents * 2.0
            }

            /// The smallest box containing both `self` and `other`.
            pub fn encapsulate(&self, other: &Self) -> Self {
                let min = self.min().min(other.min());
                let max = self.max().max(other.max());

                Self {
                    center: (min + max) * 0.5,
                    extents: (max - min) * 0.5,
                }
            }
        }
    };
}

pub(crate) use impl_ty_bounds;
