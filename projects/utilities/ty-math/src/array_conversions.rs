/// Generates the array and slice conversions shared by the flat vector-like
/// types: `to_array` / `from_array`, `from_slice` / `write_to_slice`, and
/// `From<[T; N]>`. The concrete form also emits the reverse `From<Self>`, which
/// the orphan rule forbids for a generic component type; use `to_array` there.
///
/// # Arguments
/// The generic form takes the type name, the component count, and the fields in
/// order. The concrete form inserts the element type before the count.
macro_rules! ty_array_conversions {
    ($type:ident, $count:literal, $($field:ident),+ $(,)?) => {
        impl<T: Copy> $type<T> {
            /// The components as an array.
            pub fn to_array(&self) -> [T; $count] {
                [$(self.$field),+]
            }

            /// A value from an array of components.
            pub fn from_array(array: [T; $count]) -> Self {
                let [$($field),+] = array;
                Self { $($field),+ }
            }

            /// A value read from the first components of `slice`.
            ///
            /// # Panics
            /// Panics when `slice` holds fewer than the component count.
            pub fn from_slice(slice: &[T]) -> Self {
                let array: [T; $count] = slice
                    .get(..$count)
                    .expect("from_slice: slice holds fewer than the component count")
                    .try_into()
                    .unwrap();
                Self::from_array(array)
            }

            /// Writes the components into the start of `slice`.
            ///
            /// # Panics
            /// Panics when `slice` holds fewer than the component count.
            pub fn write_to_slice(&self, slice: &mut [T]) {
                slice[..$count].copy_from_slice(&self.to_array());
            }
        }

        impl<T: Copy> From<[T; $count]> for $type<T> {
            fn from(array: [T; $count]) -> Self {
                Self::from_array(array)
            }
        }
    };

    ($type:ident, $element:ty, $count:literal, $($field:ident),+ $(,)?) => {
        impl $type {
            /// The components as an array.
            pub fn to_array(&self) -> [$element; $count] {
                [$(self.$field),+]
            }

            /// A value from an array of components.
            pub fn from_array(array: [$element; $count]) -> Self {
                let [$($field),+] = array;
                Self { $($field),+ }
            }

            /// A value read from the first components of `slice`.
            ///
            /// # Panics
            /// Panics when `slice` holds fewer than the component count.
            pub fn from_slice(slice: &[$element]) -> Self {
                let array: [$element; $count] = slice
                    .get(..$count)
                    .expect("from_slice: slice holds fewer than the component count")
                    .try_into()
                    .unwrap();
                Self::from_array(array)
            }

            /// Writes the components into the start of `slice`.
            ///
            /// # Panics
            /// Panics when `slice` holds fewer than the component count.
            pub fn write_to_slice(&self, slice: &mut [$element]) {
                slice[..$count].copy_from_slice(&self.to_array());
            }
        }

        impl From<[$element; $count]> for $type {
            fn from(array: [$element; $count]) -> Self {
                Self::from_array(array)
            }
        }

        impl From<$type> for [$element; $count] {
            fn from(value: $type) -> Self {
                value.to_array()
            }
        }
    };
}

pub(crate) use ty_array_conversions;
