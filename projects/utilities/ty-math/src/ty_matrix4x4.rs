use crate::{TyQuaternion, TyVector3, TyVector4};
use std::ops::Mul;

/// A 4x4 matrix with component type `T`, stored as four column-major
/// [`TyVector4`] columns. In a transform matrix `columns[3]` is the translation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TyMatrix4x4<T = f64> {
    /// The columns in column-major order; `columns[col]` holds rows `0..4` of
    /// column `col`.
    pub columns: [TyVector4<T>; 4],
}

impl<T> TyMatrix4x4<T> {
    /// Creates a matrix from its four columns, in column-major order.
    pub fn new(columns: [TyVector4<T>; 4]) -> Self {
        Self { columns }
    }
}

impl<T: Copy> TyMatrix4x4<T> {
    /// Creates a matrix from four column arrays in column-major order, indexed
    /// `arrays[col][row]`. This is glTF's node-matrix layout.
    pub fn from_column_arrays(arrays: [[T; 4]; 4]) -> Self {
        Self {
            columns: arrays.map(TyVector4::from_array),
        }
    }

    /// The four columns as `[[T; 4]; 4]` in column-major order,
    /// indexed `[col][row]`.
    pub fn to_column_arrays(&self) -> [[T; 4]; 4] {
        self.columns.map(|column| column.to_array())
    }

    /// The element at `row` and `col`.
    ///
    /// # Panics
    /// Panics when `row` or `col` is not in `0..4`.
    pub fn get(&self, row: usize, col: usize) -> T {
        self.columns[col].component(row)
    }
}

/// Implements the float-only matrix operations for a concrete floating-point
/// component type.
macro_rules! impl_ty_matrix4x4_float {
    ($t:ty) => {
        impl TyMatrix4x4<$t> {
            /// The identity matrix.
            pub fn identity() -> Self {
                Self {
                    columns: [
                        TyVector4::new(1.0, 0.0, 0.0, 0.0),
                        TyVector4::new(0.0, 1.0, 0.0, 0.0),
                        TyVector4::new(0.0, 0.0, 1.0, 0.0),
                        TyVector4::new(0.0, 0.0, 0.0, 1.0),
                    ],
                }
            }

            /// The rotation matrix of a unit `rotation`, its upper-left 3x3 the
            /// rotation and the rest identity.
            pub fn from_quaternion(rotation: TyQuaternion<$t>) -> Self {
                let TyQuaternion { x, y, z, w } = rotation;

                let x2 = x * 2.0;
                let y2 = y * 2.0;
                let z2 = z * 2.0;

                let xx = x * x2;
                let yy = y * y2;
                let zz = z * z2;
                let xy = x * y2;
                let xz = x * z2;
                let yz = y * z2;
                let wx = w * x2;
                let wy = w * y2;
                let wz = w * z2;

                Self {
                    columns: [
                        TyVector4::new(1.0 - (yy + zz), xy + wz, xz - wy, 0.0),
                        TyVector4::new(xy - wz, 1.0 - (xx + zz), yz + wx, 0.0),
                        TyVector4::new(xz + wy, yz - wx, 1.0 - (xx + yy), 0.0),
                        TyVector4::new(0.0, 0.0, 0.0, 1.0),
                    ],
                }
            }

            /// Transforms `point` as `[x, y, z, 1]`, applying the rotation, scale,
            /// and translation of an affine transform matrix and dropping the
            /// resulting `w`.
            pub fn transform_point(&self, point: TyVector3<$t>) -> TyVector3<$t> {
                let [c0, c1, c2, c3] = self.columns;
                (c0 * point.x + c1 * point.y + c2 * point.z + c3).truncate()
            }

            /// Transforms `point` as `[x, y, z, 0]`, applying the rotation and
            /// scale but not the translation, for a direction rather than a
            /// position.
            pub fn transform_vector(&self, vector: TyVector3<$t>) -> TyVector3<$t> {
                let [c0, c1, c2, _] = self.columns;
                (c0 * vector.x + c1 * vector.y + c2 * vector.z).truncate()
            }
        }

        impl Default for TyMatrix4x4<$t> {
            fn default() -> Self {
                Self::identity()
            }
        }

        impl Mul<TyVector4<$t>> for TyMatrix4x4<$t> {
            type Output = TyVector4<$t>;

            /// The matrix-vector product, a linear combination of the columns.
            fn mul(self, rhs: TyVector4<$t>) -> TyVector4<$t> {
                let [c0, c1, c2, c3] = self.columns;
                c0 * rhs.x + c1 * rhs.y + c2 * rhs.z + c3 * rhs.w
            }
        }

        impl Mul for TyMatrix4x4<$t> {
            type Output = Self;

            /// The matrix product `self * rhs`. Applied to a vector, the result
            /// applies `rhs` first, then `self`.
            fn mul(self, rhs: Self) -> Self {
                Self {
                    columns: rhs.columns.map(|column| self * column),
                }
            }
        }
    };
}

impl_ty_matrix4x4_float!(f32);
impl_ty_matrix4x4_float!(f64);

#[cfg(test)]
mod tests {
    use crate::{TyMatrix4x4F64, TyQuaternionF64, TyVector3F64, TyVector4F64};
    use std::f64::consts::PI;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn close_vec(a: TyVector3F64, b: TyVector3F64) -> bool {
        close(a.x, b.x) && close(a.y, b.y) && close(a.z, b.z)
    }

    #[test]
    fn identity_leaves_a_point_unchanged() {
        let point = TyVector3F64::new(1.0, 2.0, 3.0);
        assert!(close_vec(
            TyMatrix4x4F64::identity().transform_point(point),
            point
        ));
    }

    #[test]
    fn identity_times_a_matrix_is_that_matrix() {
        let matrix = TyMatrix4x4F64::from_column_arrays([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ]);
        assert_eq!(TyMatrix4x4F64::identity() * matrix, matrix);
        assert_eq!(matrix * TyMatrix4x4F64::identity(), matrix);
    }

    #[test]
    fn get_reads_row_and_column() {
        // from_column_arrays takes columns, so arrays[col][row].
        let matrix = TyMatrix4x4F64::from_column_arrays([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ]);
        assert_eq!(matrix.get(0, 0), 1.0);
        assert_eq!(matrix.get(1, 0), 2.0);
        assert_eq!(matrix.get(0, 3), 13.0);
        assert_eq!(matrix.get(3, 3), 16.0);
    }

    #[test]
    fn from_quaternion_matches_quaternion_rotation() {
        // The rotation matrix of a quaternion transforms a point exactly as the
        // quaternion rotates it.
        let quaternion = TyQuaternionF64::from_axis_angle(TyVector3F64::new(1.0, 2.0, 3.0), 0.9);
        let matrix = TyMatrix4x4F64::from_quaternion(quaternion);
        for point in [
            TyVector3F64::new(1.0, 0.0, 0.0),
            TyVector3F64::new(0.0, 1.0, 0.0),
            TyVector3F64::new(0.0, 0.0, 1.0),
            TyVector3F64::new(1.0, -2.0, 3.0),
        ] {
            assert!(close_vec(
                matrix.transform_point(point),
                quaternion.rotate(point)
            ));
        }
    }

    #[test]
    fn matrix_product_composes_transforms() {
        // A quarter turn about z carries the translation +x child into world +y.
        let rotate = TyMatrix4x4F64::from_quaternion(TyQuaternionF64::from_axis_angle(
            TyVector3F64::new(0.0, 0.0, 1.0),
            PI / 2.0,
        ));
        let translate = TyMatrix4x4F64::new([
            TyVector4F64::new(1.0, 0.0, 0.0, 0.0),
            TyVector4F64::new(0.0, 1.0, 0.0, 0.0),
            TyVector4F64::new(0.0, 0.0, 1.0, 0.0),
            TyVector4F64::new(1.0, 0.0, 0.0, 1.0),
        ]);
        let composed = rotate * translate;
        assert!(close_vec(
            composed.transform_point(TyVector3F64::new(0.0, 0.0, 0.0)),
            TyVector3F64::new(0.0, 1.0, 0.0)
        ));
    }

    #[test]
    fn transform_vector_ignores_translation() {
        let translate = TyMatrix4x4F64::new([
            TyVector4F64::new(1.0, 0.0, 0.0, 0.0),
            TyVector4F64::new(0.0, 1.0, 0.0, 0.0),
            TyVector4F64::new(0.0, 0.0, 1.0, 0.0),
            TyVector4F64::new(5.0, 6.0, 7.0, 1.0),
        ]);
        let vector = TyVector3F64::new(1.0, 2.0, 3.0);
        assert!(close_vec(translate.transform_vector(vector), vector));
    }

    #[test]
    fn column_arrays_round_trip() {
        let arrays = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ];
        assert_eq!(
            TyMatrix4x4F64::from_column_arrays(arrays).to_column_arrays(),
            arrays
        );
    }
}
