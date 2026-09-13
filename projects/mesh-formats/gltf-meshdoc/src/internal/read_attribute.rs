use crate::{Error, Result};
use gltf::{
    Accessor, Buffer,
    accessor::{DataType, Dimensions, Item, Iter},
};
use meshdoc::MeshAttributeComponents;

/// A further vertex attribute's components with its width: the accessor's
/// elements flattened. Unsigned 8- and 16-bit integers keep their type;
/// floats, the signed and 32-bit integers, and integers the accessor
/// normalizes read as floats.
pub fn read_attribute<'a, 's, F>(
    accessor: Accessor<'a>,
    get: F,
) -> Result<(usize, MeshAttributeComponents)>
where
    F: Clone + Fn(Buffer<'a>) -> Option<&'s [u8]>,
{
    let width = match accessor.dimensions() {
        Dimensions::Scalar => 1,
        Dimensions::Vec2 => 2,
        Dimensions::Vec3 => 3,
        Dimensions::Vec4 | Dimensions::Mat2 => 4,
        Dimensions::Mat3 => 9,
        Dimensions::Mat4 => 16,
    };

    let normalized = accessor.normalized();
    let dimensions = accessor.dimensions();

    let components = match accessor.data_type() {
        DataType::F32 => MeshAttributeComponents::F64(
            read_elements::<f32, _>(accessor, dimensions, get)?
                .into_iter()
                .map(f64::from)
                .collect(),
        ),
        DataType::U8 if !normalized => {
            MeshAttributeComponents::U8(read_elements::<u8, _>(accessor, dimensions, get)?)
        }
        DataType::U8 => MeshAttributeComponents::F64(
            read_elements::<u8, _>(accessor, dimensions, get)?
                .into_iter()
                .map(|value| normalize(f64::from(value), f64::from(u8::MAX)))
                .collect(),
        ),
        DataType::I8 => MeshAttributeComponents::F64(
            read_elements::<i8, _>(accessor, dimensions, get)?
                .into_iter()
                .map(|value| {
                    if normalized {
                        normalize(f64::from(value), f64::from(i8::MAX))
                    } else {
                        f64::from(value)
                    }
                })
                .collect(),
        ),
        DataType::U16 if !normalized => {
            MeshAttributeComponents::U16(read_elements::<u16, _>(accessor, dimensions, get)?)
        }
        DataType::U16 => MeshAttributeComponents::F64(
            read_elements::<u16, _>(accessor, dimensions, get)?
                .into_iter()
                .map(|value| normalize(f64::from(value), f64::from(u16::MAX)))
                .collect(),
        ),
        DataType::I16 => MeshAttributeComponents::F64(
            read_elements::<i16, _>(accessor, dimensions, get)?
                .into_iter()
                .map(|value| {
                    if normalized {
                        normalize(f64::from(value), f64::from(i16::MAX))
                    } else {
                        f64::from(value)
                    }
                })
                .collect(),
        ),
        DataType::U32 => MeshAttributeComponents::F64(
            read_elements::<u32, _>(accessor, dimensions, get)?
                .into_iter()
                .map(f64::from)
                .collect(),
        ),
    };

    Ok((width, components))
}

/// The accessor's elements of `T` components, flattened, iterated at the
/// element width `dimensions` gives.
fn read_elements<'a, 's, T, F>(
    accessor: Accessor<'a>,
    dimensions: Dimensions,
    get: F,
) -> Result<Vec<T>>
where
    T: Item + Copy,
    F: Clone + Fn(Buffer<'a>) -> Option<&'s [u8]>,
{
    let missing = || Error::invalid("a vertex attribute accessor has no buffer data");

    Ok(match dimensions {
        Dimensions::Scalar => Iter::<T>::new(accessor, get).ok_or_else(missing)?.collect(),
        Dimensions::Vec2 => Iter::<[T; 2]>::new(accessor, get)
            .ok_or_else(missing)?
            .flatten()
            .collect(),
        Dimensions::Vec3 => Iter::<[T; 3]>::new(accessor, get)
            .ok_or_else(missing)?
            .flatten()
            .collect(),
        Dimensions::Vec4 => Iter::<[T; 4]>::new(accessor, get)
            .ok_or_else(missing)?
            .flatten()
            .collect(),
        Dimensions::Mat2 => Iter::<[[T; 2]; 2]>::new(accessor, get)
            .ok_or_else(missing)?
            .flatten()
            .flatten()
            .collect(),
        Dimensions::Mat3 => Iter::<[[T; 3]; 3]>::new(accessor, get)
            .ok_or_else(missing)?
            .flatten()
            .flatten()
            .collect(),
        Dimensions::Mat4 => Iter::<[[T; 4]; 4]>::new(accessor, get)
            .ok_or_else(missing)?
            .flatten()
            .flatten()
            .collect(),
    })
}

/// `value` divided by `max`, clamped at `-1` for a signed type as the spec
/// says.
fn normalize(value: f64, max: f64) -> f64 {
    (value / max).max(-1.0)
}
