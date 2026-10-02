use crate::BVoxValuePoolValue;
use branded_id::{
    U32Id,
    soa::{IdField, IdStruct},
};
use std::fmt::{Debug, Formatter, Result as FmtResult};

/// One kind's values in a [`VoxValuePool`](crate::VoxValuePool), read by
/// value id without a match per value.
///
/// Reach one with [`VoxValuePool::values`](crate::VoxValuePool::values) or a
/// per-kind accessor such as
/// [`VoxValuePool::float_values`](crate::VoxValuePool::float_values).
pub struct VoxValueColumn<'a, T> {
    value_ids: &'a IdStruct<BVoxValuePoolValue>,

    /// The value pool's column, keyed by `value_ids`.
    values: &'a IdField<BVoxValuePoolValue, T>,
}

impl<'a, T> VoxValueColumn<'a, T> {
    /// # Safety
    /// `values` holds a value for every id `value_ids` retains.
    pub(crate) unsafe fn new(
        value_ids: &'a IdStruct<BVoxValuePoolValue>,
        values: &'a IdField<BVoxValuePoolValue, T>,
    ) -> Self {
        Self { value_ids, values }
    }

    /// The value at `id`, or `None` if `id` is not one of the value pool's
    /// values.
    pub fn get(&self, id: U32Id<BVoxValuePoolValue>) -> Option<&'a T> {
        // Safety: a retained id has a value in the column.
        self.value_ids
            .is_retained(id)
            .then(|| unsafe { self.values.get(id) })
    }

    /// Values in listing order, as `(id, value)`.
    pub fn iter(&self) -> impl Iterator<Item = (U32Id<BVoxValuePoolValue>, &'a T)> + use<'a, T> {
        // Safety: the column holds a value for every id in its id pool.
        let values = unsafe { self.values.iter(self.value_ids) };
        self.value_ids.iter().zip(values)
    }
}

impl<T> Clone for VoxValueColumn<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for VoxValueColumn<'_, T> {}

impl<T: Debug> Debug for VoxValueColumn<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T: PartialEq> PartialEq for VoxValueColumn<'_, T> {
    /// Compares values in listing order. Ids do not take part.
    fn eq(&self, other: &Self) -> bool {
        self.iter()
            .map(|(_, value)| value)
            .eq(other.iter().map(|(_, value)| value))
    }
}

#[cfg(test)]
mod tests {
    use crate::{BVoxValuePoolValue, VoxValuePool};
    use branded_id::U32Id;

    fn value_id(index: u32) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index)
    }

    #[test]
    fn a_column_reads_by_id_and_in_listing_order() {
        let mut value_pool = VoxValuePool::int(vec![10, 20, 30]).unwrap();
        value_pool.move_value(value_id(2), 0).unwrap();

        let ints = value_pool.int_values().unwrap();

        assert_eq!(ints.get(value_id(1)), Some(&20));
        assert_eq!(ints.get(value_id(3)), None);

        assert_eq!(
            ints.iter().collect::<Vec<_>>(),
            [(value_id(2), &30), (value_id(0), &10), (value_id(1), &20)]
        );
    }
}
