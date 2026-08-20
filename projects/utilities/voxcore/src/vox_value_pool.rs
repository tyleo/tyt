use crate::{BVoxValuePoolValue, Error, Result, VoxValue, VoxValuePoolKind, VoxValuePoolValueRef};
use branded_id::{
    U32Id,
    soa::{IdField, IdRemap, IdStruct},
};
use std::mem;

/// A shared value pool: one [`VoxValuePoolKind`] column keyed by an id pool of
/// value ids.
///
/// Build a value pool with the constructor for its kind (for example
/// [`float`](Self::float) or [`vec_4_float`](Self::vec_4_float)), which checks
/// its input and retains one id per value in the given order. Read values back
/// by id with [`value`](Self::value) or in listing order with
/// [`iter_values`](Self::iter_values), and match [`kind`](Self::kind) for the
/// kind.
#[derive(Debug)]
pub struct VoxValuePool {
    /// Value id pool. Its listing order is the value pool's value order.
    value_ids: IdStruct<BVoxValuePoolValue>,

    /// The kind and its typed value column, keyed by `value_ids`.
    kind: VoxValuePoolKind,
}

impl VoxValuePool {
    /// Creates a `bool` value pool holding `values`, retaining ids in order.
    pub fn boolean(values: Vec<bool>) -> Self {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Bool(values),
        }
    }

    /// Creates a `float` value pool holding `values`, retaining ids in order.
    /// Errors, building nothing, if a value is NaN.
    pub fn float(values: Vec<f64>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Float(values),
        }
        .checked()
    }

    /// Creates an `int` value pool holding `values`, retaining ids in order.
    /// Errors, building nothing, if a value's magnitude exceeds `2^53 - 1`.
    pub fn int(values: Vec<i64>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Int(values),
        }
        .checked()
    }

    /// Creates a `json` value pool holding `values`, retaining ids in order.
    pub fn json(values: Vec<VoxValue>) -> Self {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Json(values),
        }
    }

    /// Creates a `string` value pool holding `values`, retaining ids in order.
    pub fn string(values: Vec<String>) -> Self {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::String(values),
        }
    }

    /// Creates a `vec-2-float` value pool holding `values`, retaining ids in
    /// order. Errors, building nothing, if a component is NaN.
    pub fn vec_2_float(values: Vec<[f64; 2]>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Vec2Float(values),
        }
        .checked()
    }

    /// Creates a `vec-2-int` value pool holding `values`, retaining ids in
    /// order. Errors, building nothing, if a component's magnitude exceeds
    /// `2^53 - 1`.
    pub fn vec_2_int(values: Vec<[i64; 2]>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Vec2Int(values),
        }
        .checked()
    }

    /// Creates a `vec-3-float` value pool holding `values`, retaining ids in
    /// order. Errors, building nothing, if a component is NaN.
    pub fn vec_3_float(values: Vec<[f64; 3]>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Vec3Float(values),
        }
        .checked()
    }

    /// Creates a `vec-3-int` value pool holding `values`, retaining ids in
    /// order. Errors, building nothing, if a component's magnitude exceeds
    /// `2^53 - 1`.
    pub fn vec_3_int(values: Vec<[i64; 3]>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Vec3Int(values),
        }
        .checked()
    }

    /// Creates a `vec-4-float` value pool holding `values`, retaining ids in
    /// order. Errors, building nothing, if a component is NaN.
    pub fn vec_4_float(values: Vec<[f64; 4]>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Vec4Float(values),
        }
        .checked()
    }

    /// Creates a `vec-4-int` value pool holding `values`, retaining ids in
    /// order. Errors, building nothing, if a component's magnitude exceeds
    /// `2^53 - 1`.
    pub fn vec_4_int(values: Vec<[i64; 4]>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Vec4Int(values),
        }
        .checked()
    }

    /// Maps a freshly built value pool's first out-of-domain value onto the
    /// creation error.
    fn checked(self) -> Result<Self> {
        match self.first_out_of_domain_value() {
            None => Ok(self),
            Some(value_id) => Err(Error::MalformedValuePoolValue { value_id }),
        }
    }

    /// Deep copy. Liveness lives in the id pool, so the column can't derive
    /// `Clone`. Rebuild it against the cloned id pool.
    pub fn clone_value_pool(&self) -> Self {
        let kind = match &self.kind {
            VoxValuePoolKind::Bool(values) => {
                VoxValuePoolKind::Bool(cloned(&self.value_ids, values))
            }
            VoxValuePoolKind::Float(values) => {
                VoxValuePoolKind::Float(cloned(&self.value_ids, values))
            }
            VoxValuePoolKind::Int(values) => VoxValuePoolKind::Int(cloned(&self.value_ids, values)),
            VoxValuePoolKind::Json(values) => {
                VoxValuePoolKind::Json(cloned(&self.value_ids, values))
            }
            VoxValuePoolKind::String(values) => {
                VoxValuePoolKind::String(cloned(&self.value_ids, values))
            }
            VoxValuePoolKind::Vec2Float(values) => {
                VoxValuePoolKind::Vec2Float(cloned(&self.value_ids, values))
            }
            VoxValuePoolKind::Vec2Int(values) => {
                VoxValuePoolKind::Vec2Int(cloned(&self.value_ids, values))
            }
            VoxValuePoolKind::Vec3Float(values) => {
                VoxValuePoolKind::Vec3Float(cloned(&self.value_ids, values))
            }
            VoxValuePoolKind::Vec3Int(values) => {
                VoxValuePoolKind::Vec3Int(cloned(&self.value_ids, values))
            }
            VoxValuePoolKind::Vec4Float(values) => {
                VoxValuePoolKind::Vec4Float(cloned(&self.value_ids, values))
            }
            VoxValuePoolKind::Vec4Int(values) => {
                VoxValuePoolKind::Vec4Int(cloned(&self.value_ids, values))
            }
        };

        Self {
            value_ids: self.value_ids.clone(),
            kind,
        }
    }

    /// The kind, for matching. Read the values through [`value`](Self::value)
    /// and [`iter_values`](Self::iter_values).
    pub fn kind(&self) -> &VoxValuePoolKind {
        &self.kind
    }

    /// Releases value `id`, keeping the surviving values' listing order. The id
    /// must be one of this value pool's values. The caller repoints any palette
    /// cell drawing it first.
    pub(crate) fn release_value_stable(&mut self, id: U32Id<BVoxValuePoolValue>) {
        // Safety: the id is retained, so it has a value in the column.
        unsafe {
            match &mut self.kind {
                VoxValuePoolKind::Bool(values) => values.release(id),
                VoxValuePoolKind::Float(values) => values.release(id),
                VoxValuePoolKind::Int(values) => values.release(id),
                VoxValuePoolKind::Json(values) => values.release(id),
                VoxValuePoolKind::String(values) => values.release(id),
                VoxValuePoolKind::Vec2Float(values) => values.release(id),
                VoxValuePoolKind::Vec2Int(values) => values.release(id),
                VoxValuePoolKind::Vec3Float(values) => values.release(id),
                VoxValuePoolKind::Vec3Int(values) => values.release(id),
                VoxValuePoolKind::Vec4Float(values) => values.release(id),
                VoxValuePoolKind::Vec4Int(values) => values.release(id),
            }
        }
        self.value_ids.release_stable(id);
    }

    /// Whether `id` is one of this value pool's values.
    pub fn contains_value(&self, id: U32Id<BVoxValuePoolValue>) -> bool {
        self.value_ids.is_retained(id)
    }

    /// The id of the first value outside its kind's value domain, or `None`
    /// if every value is within it. The checked constructors gate on this, so
    /// a flaw found later is a voxcore bug;
    /// [`VoxMain::validate`](crate::VoxMain::validate) audits for one anyway.
    pub(crate) fn first_out_of_domain_value(&self) -> Option<U32Id<BVoxValuePoolValue>> {
        for (value_id, value) in self.iter_values() {
            let in_domain = match value {
                VoxValuePoolValueRef::Float(value) => float_in_domain(value),
                VoxValuePoolValueRef::Int(value) => int_in_domain(value),
                VoxValuePoolValueRef::Vec2Float(components) => floats_in_domain(components),
                VoxValuePoolValueRef::Vec2Int(components) => ints_in_domain(components),
                VoxValuePoolValueRef::Vec3Float(components) => floats_in_domain(components),
                VoxValuePoolValueRef::Vec3Int(components) => ints_in_domain(components),
                VoxValuePoolValueRef::Vec4Float(components) => floats_in_domain(components),
                VoxValuePoolValueRef::Vec4Int(components) => ints_in_domain(components),
                VoxValuePoolValueRef::Bool(_)
                | VoxValuePoolValueRef::Json(_)
                | VoxValuePoolValueRef::String(_) => true,
            };
            if !in_domain {
                return Some(value_id);
            }
        }
        None
    }

    /// Compacts the value id pool back to a contiguous `0..len` in listing
    /// order and returns the relabeling, so the caller can translate the
    /// palette cells that point at these values.
    pub(crate) fn gc_values(&mut self) -> IdRemap<BVoxValuePoolValue, u32> {
        let remap = self.value_ids.gc();
        // Safety: the column was in sync with the pre-gc id pool, and nothing
        // has retained or released since.
        unsafe {
            match &mut self.kind {
                VoxValuePoolKind::Bool(values) => values.gc(&remap),
                VoxValuePoolKind::Float(values) => values.gc(&remap),
                VoxValuePoolKind::Int(values) => values.gc(&remap),
                VoxValuePoolKind::Json(values) => values.gc(&remap),
                VoxValuePoolKind::String(values) => values.gc(&remap),
                VoxValuePoolKind::Vec2Float(values) => values.gc(&remap),
                VoxValuePoolKind::Vec2Int(values) => values.gc(&remap),
                VoxValuePoolKind::Vec3Float(values) => values.gc(&remap),
                VoxValuePoolKind::Vec3Int(values) => values.gc(&remap),
                VoxValuePoolKind::Vec4Float(values) => values.gc(&remap),
                VoxValuePoolKind::Vec4Int(values) => values.gc(&remap),
            }
        }
        remap
    }

    /// Whether the value pool holds no values.
    pub fn is_empty(&self) -> bool {
        self.value_ids.is_empty()
    }

    /// Values in listing order, as `(id, value)`.
    pub fn iter_values(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxValuePoolValue>, VoxValuePoolValueRef<'_>)> + '_ {
        self.value_ids
            .iter()
            .map(move |value_id| (value_id, self.value_ref(value_id)))
    }

    /// The number of values in the value pool, across every kind.
    pub fn len(&self) -> usize {
        self.value_ids.len()
    }

    /// Moves value `id` to listing position `index`, shifting the values
    /// between its old and new positions one slot. Errors, changing nothing, if
    /// `id` is not one of this value pool's values or `index` is at or past
    /// [`len`](Self::len).
    pub fn move_value(&mut self, id: U32Id<BVoxValuePoolValue>, index: usize) -> Result<()> {
        if !self.value_ids.is_retained(id) {
            return Err(Error::UnknownValuePoolValue { value_id: id });
        }
        let count = self.value_ids.len();
        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }
        self.value_ids.move_to(id, index);
        Ok(())
    }

    /// Rewrites the listing order to `new_order_ids`. `None`, changing nothing,
    /// if `new_order_ids` does not list every value id exactly once.
    pub(crate) fn set_value_order(
        &mut self,
        new_order_ids: &[U32Id<BVoxValuePoolValue>],
    ) -> Option<()> {
        self.value_ids.try_set_order(new_order_ids)
    }

    /// The value at `id`, typed by the value pool's kind, or `None` if `id` is
    /// not one of this value pool's values.
    pub fn value(&self, id: U32Id<BVoxValuePoolValue>) -> Option<VoxValuePoolValueRef<'_>> {
        self.value_ids.is_retained(id).then(|| self.value_ref(id))
    }

    /// The listing position of value `id`, or `None` if `id` is not one of this
    /// value pool's values.
    pub fn value_index(&self, id: U32Id<BVoxValuePoolValue>) -> Option<usize> {
        self.value_ids.index_of(id)
    }

    /// The typed ref for a retained `id`.
    fn value_ref(&self, id: U32Id<BVoxValuePoolValue>) -> VoxValuePoolValueRef<'_> {
        // Safety: the id is retained, so it has a value in the column.
        unsafe {
            match &self.kind {
                VoxValuePoolKind::Bool(values) => VoxValuePoolValueRef::Bool(*values.get(id)),
                VoxValuePoolKind::Float(values) => VoxValuePoolValueRef::Float(*values.get(id)),
                VoxValuePoolKind::Int(values) => VoxValuePoolValueRef::Int(*values.get(id)),
                VoxValuePoolKind::Json(values) => VoxValuePoolValueRef::Json(values.get(id)),
                VoxValuePoolKind::String(values) => VoxValuePoolValueRef::String(values.get(id)),
                VoxValuePoolKind::Vec2Float(values) => {
                    VoxValuePoolValueRef::Vec2Float(values.get(id))
                }
                VoxValuePoolKind::Vec2Int(values) => VoxValuePoolValueRef::Vec2Int(values.get(id)),
                VoxValuePoolKind::Vec3Float(values) => {
                    VoxValuePoolValueRef::Vec3Float(values.get(id))
                }
                VoxValuePoolKind::Vec3Int(values) => VoxValuePoolValueRef::Vec3Int(values.get(id)),
                VoxValuePoolKind::Vec4Float(values) => {
                    VoxValuePoolValueRef::Vec4Float(values.get(id))
                }
                VoxValuePoolKind::Vec4Int(values) => VoxValuePoolValueRef::Vec4Int(values.get(id)),
            }
        }
    }
}

impl Drop for VoxValuePool {
    fn drop(&mut self) {
        // Safety: the column holds a value for every id in the id pool.
        unsafe {
            match &mut self.kind {
                VoxValuePoolKind::Bool(values) => values.release_all(&self.value_ids),
                VoxValuePoolKind::Float(values) => values.release_all(&self.value_ids),
                VoxValuePoolKind::Int(values) => values.release_all(&self.value_ids),
                VoxValuePoolKind::Json(values) => values.release_all(&self.value_ids),
                VoxValuePoolKind::String(values) => values.release_all(&self.value_ids),
                VoxValuePoolKind::Vec2Float(values) => values.release_all(&self.value_ids),
                VoxValuePoolKind::Vec2Int(values) => values.release_all(&self.value_ids),
                VoxValuePoolKind::Vec3Float(values) => values.release_all(&self.value_ids),
                VoxValuePoolKind::Vec3Int(values) => values.release_all(&self.value_ids),
                VoxValuePoolKind::Vec4Float(values) => values.release_all(&self.value_ids),
                VoxValuePoolKind::Vec4Int(values) => values.release_all(&self.value_ids),
            }
        }
    }
}

impl PartialEq for VoxValuePool {
    /// Compares kind and values in listing order. The id labels underneath the
    /// listing do not take part.
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(&self.kind) == mem::discriminant(&other.kind)
            && self
                .iter_values()
                .map(|(_, value)| value)
                .eq(other.iter_values().map(|(_, value)| value))
    }
}

/// The largest magnitude an int value may carry: `2^53 - 1`, the widest span a
/// consumer reading numbers as doubles keeps exact.
const MAX_INT_MAGNITUDE: i64 = (1 << 53) - 1;

/// Whether `value` is within the float value domain: a finite number or an
/// infinity.
fn float_in_domain(value: f64) -> bool {
    !value.is_nan()
}

/// Whether every component of a float vector is within the float value domain.
fn floats_in_domain(components: &[f64]) -> bool {
    components
        .iter()
        .all(|&component| float_in_domain(component))
}

/// Whether `value` is within the int value domain: magnitude at most
/// [`MAX_INT_MAGNITUDE`].
fn int_in_domain(value: i64) -> bool {
    (-MAX_INT_MAGNITUDE..=MAX_INT_MAGNITUDE).contains(&value)
}

/// Whether every component of an int vector is within the int value domain.
fn ints_in_domain(components: &[i64]) -> bool {
    components.iter().all(|&component| int_in_domain(component))
}

/// Builds the paired id pool and value column for `values`, retaining ids in
/// order.
fn columns<T>(values: Vec<T>) -> (IdStruct<BVoxValuePoolValue>, IdField<BVoxValuePoolValue, T>) {
    let mut ids = IdStruct::new();
    let mut column = IdField::with_capacity(values.len());
    for value in values {
        let value_id = ids.retain();
        column.retain(value_id, value);
    }
    (ids, column)
}

/// Clones the values retained in `column` against the same ids.
fn cloned<T: Clone>(
    ids: &IdStruct<BVoxValuePoolValue>,
    column: &IdField<BVoxValuePoolValue, T>,
) -> IdField<BVoxValuePoolValue, T> {
    let mut copy = IdField::with_capacity(ids.len());
    for value_id in ids.iter() {
        // Safety: retained ids have a value.
        copy.retain(value_id, unsafe { column.get(value_id) }.clone());
    }
    copy
}

#[cfg(test)]
mod tests {
    use crate::{BVoxValuePoolValue, Error, VoxValuePool, VoxValuePoolValueRef};
    use branded_id::U32Id;

    fn value_id(index: u32) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index)
    }

    #[test]
    fn float_value_pool_reads_back_in_order() {
        let value_pool = VoxValuePool::float(vec![0.0, 0.5, 1.0]).unwrap();

        assert_eq!(value_pool.len(), 3);
        let values: Vec<_> = value_pool.iter_values().collect();
        assert_eq!(values.len(), 3);
        assert_eq!(
            values[1],
            (U32Id::from_u32(1), VoxValuePoolValueRef::Float(0.5))
        );
        assert_eq!(
            value_pool.value(U32Id::from_u32(2)),
            Some(VoxValuePoolValueRef::Float(1.0))
        );
        assert_eq!(value_pool.value(U32Id::from_u32(3)), None);
    }

    #[test]
    fn vector_value_pool_holds_typed_components() {
        let value_pool = VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap();

        assert_eq!(value_pool.len(), 1);
        assert_eq!(
            value_pool.value(U32Id::from_u32(0)),
            Some(VoxValuePoolValueRef::Vec4Float(&[1.0, 0.0, 0.0, 1.0]))
        );
    }

    #[test]
    fn value_pools_compare_by_kind_and_ordered_values() {
        let a = VoxValuePool::int(vec![1, 2]).unwrap();
        let mut b = VoxValuePool::int(vec![2, 1]).unwrap();
        assert_ne!(a, b);

        // Moving b's values into a's order makes the value pools equal, even
        // though their id labels now differ per position.
        b.move_value(U32Id::from_u32(1), 0).unwrap();
        assert_eq!(a, b);

        // Same numbers, different kind.
        let c = VoxValuePool::float(vec![1.0, 2.0]).unwrap();
        assert_ne!(a, c);
    }

    #[test]
    fn move_value_reorders_the_listing_and_validates() {
        let mut value_pool =
            VoxValuePool::string(vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]);
        let b_id = U32Id::from_u32(1);

        assert_eq!(value_pool.move_value(b_id, 2), Ok(()));
        let order: Vec<_> = value_pool
            .iter_values()
            .map(|(_, value)| match value {
                VoxValuePoolValueRef::String(text) => text.to_owned(),
                other => panic!("unexpected value {other:?}"),
            })
            .collect();
        assert_eq!(order, ["a", "c", "b"]);
        assert_eq!(value_pool.value_index(b_id), Some(2));

        // An out-of-range index and an unknown id are rejected.
        assert_eq!(
            value_pool.move_value(b_id, 3),
            Err(Error::IndexPastCount { index: 3, count: 3 })
        );
        assert_eq!(
            value_pool.move_value(U32Id::from_u32(9), 0),
            Err(Error::UnknownValuePoolValue {
                value_id: U32Id::from_u32(9)
            })
        );
    }

    #[test]
    fn clone_value_pool_is_an_independent_deep_copy() {
        let mut value_pool =
            VoxValuePool::string(vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]);
        // Hole the value pool first: equality compares values in listing order,
        // so only reading back by id catches a clone that relabels.
        value_pool.release_value_stable(U32Id::from_u32(1));

        let mut copy = value_pool.clone_value_pool();
        assert_eq!(value_pool, copy);
        assert_eq!(copy.value(U32Id::from_u32(1)), None);
        assert_eq!(
            copy.value(U32Id::from_u32(2)),
            Some(VoxValuePoolValueRef::String("c"))
        );

        copy.move_value(U32Id::from_u32(0), 1).unwrap();
        assert_ne!(value_pool, copy);
    }

    #[test]
    fn constructors_accept_an_empty_value_list() {
        assert!(VoxValuePool::boolean(vec![]).is_empty());
        assert!(VoxValuePool::vec_4_float(vec![]).unwrap().is_empty());
    }

    #[test]
    fn float_accepts_the_infinities() {
        assert!(VoxValuePool::float(vec![f64::INFINITY, f64::NEG_INFINITY, 0.5]).is_ok());
        assert!(VoxValuePool::vec_3_float(vec![[0.0, f64::INFINITY, f64::NEG_INFINITY]]).is_ok());
    }

    #[test]
    fn float_rejects_a_nan_value() {
        assert_eq!(
            VoxValuePool::float(vec![0.0, f64::NAN]).unwrap_err(),
            Error::MalformedValuePoolValue {
                value_id: value_id(1)
            }
        );
    }

    #[test]
    fn vector_float_rejects_a_nan_component() {
        assert_eq!(
            VoxValuePool::vec_3_float(vec![[0.0, 0.0, 0.0], [0.0, f64::NAN, 0.0]]).unwrap_err(),
            Error::MalformedValuePoolValue {
                value_id: value_id(1)
            }
        );
    }

    #[test]
    fn int_rejects_a_value_beyond_the_cap() {
        const MAX_MAGNITUDE: i64 = (1 << 53) - 1;
        assert!(VoxValuePool::int(vec![MAX_MAGNITUDE, -MAX_MAGNITUDE]).is_ok());
        assert_eq!(
            VoxValuePool::int(vec![0, MAX_MAGNITUDE + 1]).unwrap_err(),
            Error::MalformedValuePoolValue {
                value_id: value_id(1)
            }
        );
        assert_eq!(
            VoxValuePool::int(vec![-MAX_MAGNITUDE - 1]).unwrap_err(),
            Error::MalformedValuePoolValue {
                value_id: value_id(0)
            }
        );
    }

    #[test]
    fn vector_int_rejects_a_component_beyond_the_cap() {
        assert_eq!(
            VoxValuePool::vec_2_int(vec![[0, 1 << 53]]).unwrap_err(),
            Error::MalformedValuePoolValue {
                value_id: value_id(0)
            }
        );
    }
}
