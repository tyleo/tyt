use crate::{
    BVoxValuePoolValue, VoxValueColumn, VoxValuePool, VoxValuePoolValues,
    color::srgba_u8_from_lin_srgba_f64,
};
use branded_id::{IdVec, U32Id};
use ty_math::TyLinSrgbaF64;

/// A value pool's values as linear colors, decoded once so a read never
/// matches the kind. The kind does not mark values as colors. The caller knows
/// from the property name it resolved. A three-component value takes opaque
/// alpha.
#[derive(Clone, Debug)]
pub struct ColorValues {
    /// Colors by value id. `None` marks an id the pool does not hold.
    colors: IdVec<BVoxValuePoolValue, Option<TyLinSrgbaF64>>,
}

impl ColorValues {
    /// `value_pool`'s values as colors, or `None` when it holds no
    /// three- or four-component float vectors.
    pub fn of(value_pool: &VoxValuePool) -> Option<Self> {
        match value_pool.values() {
            VoxValuePoolValues::Vec3Float(colors) => {
                Some(decoded(colors, |&[red, green, blue]| {
                    TyLinSrgbaF64::new(red, green, blue, 1.0)
                }))
            }

            VoxValuePoolValues::Vec4Float(colors) => {
                Some(decoded(colors, |&[red, green, blue, alpha]| {
                    TyLinSrgbaF64::new(red, green, blue, alpha)
                }))
            }

            _ => None,
        }
    }

    /// The color at `value_id` as stored, or `None` if `value_id` is not one of
    /// the value pool's values.
    pub fn lin_srgba_f64(&self, value_id: U32Id<BVoxValuePoolValue>) -> Option<TyLinSrgbaF64> {
        self.colors.get(value_id.to_usize_id()).copied().flatten()
    }

    /// The color at `value_id` encoded to sRGB `[r, g, b, a]` bytes for an
    /// 8-bit consumer, or `None` if `value_id` is not one of the value pool's
    /// values.
    pub fn srgba_u8(&self, value_id: U32Id<BVoxValuePoolValue>) -> Option<[u8; 4]> {
        let color = self.lin_srgba_f64(value_id)?;
        Some(<[u8; 4]>::from(srgba_u8_from_lin_srgba_f64(color)))
    }
}

fn decoded<T>(values: VoxValueColumn<'_, T>, decode: impl Fn(&T) -> TyLinSrgbaF64) -> ColorValues {
    let mut colors = IdVec::default();
    for (value_id, value) in values.iter() {
        let slot_id = value_id.to_usize_id();
        if slot_id.to_usize() >= colors.len() {
            colors.resize(slot_id.to_usize() + 1, None);
        }

        colors[slot_id] = Some(decode(value));
    }

    ColorValues { colors }
}

#[cfg(test)]
mod tests {
    use crate::{VoxValuePool, color::ColorValues};
    use branded_id::U32Id;
    use ty_math::TyLinSrgbaF64;

    #[test]
    fn three_components_read_opaque() {
        let value_pool = VoxValuePool::vec_3_float(vec![[1.0, 0.0, 0.0]]).unwrap();
        let colors = ColorValues::of(&value_pool).unwrap();

        assert_eq!(
            colors.lin_srgba_f64(U32Id::from_u32(0)),
            Some(TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0))
        );

        assert_eq!(colors.srgba_u8(U32Id::from_u32(0)), Some([255, 0, 0, 255]));
        assert_eq!(colors.srgba_u8(U32Id::from_u32(1)), None);
    }

    #[test]
    fn a_released_value_reads_no_color() {
        let mut value_pool =
            VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 0.5], [0.0, 0.0, 1.0, 1.0]]).unwrap();

        value_pool.release_value_stable(U32Id::from_u32(0));

        let colors = ColorValues::of(&value_pool).unwrap();

        assert_eq!(colors.lin_srgba_f64(U32Id::from_u32(0)), None);

        assert_eq!(
            colors.lin_srgba_f64(U32Id::from_u32(1)),
            Some(TyLinSrgbaF64::new(0.0, 0.0, 1.0, 1.0))
        );
    }

    #[test]
    fn a_value_pool_of_another_kind_holds_no_colors() {
        let value_pool = VoxValuePool::float(vec![1.0]).unwrap();
        assert!(ColorValues::of(&value_pool).is_none());
    }
}
