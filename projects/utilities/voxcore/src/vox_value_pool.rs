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
/// [`float`](Self::float) or [`vec_4_float`](Self::vec_4_float)). Read values
/// back by id with [`value`](Self::value) or in listing order with
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
        let (value_ids, values) = checked_columns(values, float_in_domain)?;

        Ok(Self {
            value_ids,
            kind: VoxValuePoolKind::Float(values),
        })
    }

    /// Creates an `int` value pool holding `values`, retaining ids in order.
    /// Errors, building nothing, if a value's magnitude exceeds `2^53 - 1`.
    pub fn int(values: Vec<i64>) -> Result<Self> {
        let (value_ids, values) = checked_columns(values, int_in_domain)?;

        Ok(Self {
            value_ids,
            kind: VoxValuePoolKind::Int(values),
        })
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
        let (value_ids, values) = checked_columns(values, floats_in_domain)?;

        Ok(Self {
            value_ids,
            kind: VoxValuePoolKind::Vec2Float(values),
        })
    }

    /// Creates a `vec-2-int` value pool holding `values`, retaining ids in
    /// order. Errors, building nothing, if a component's magnitude exceeds
    /// `2^53 - 1`.
    pub fn vec_2_int(values: Vec<[i64; 2]>) -> Result<Self> {
        let (value_ids, values) = checked_columns(values, ints_in_domain)?;

        Ok(Self {
            value_ids,
            kind: VoxValuePoolKind::Vec2Int(values),
        })
    }

    /// Creates a `vec-3-float` value pool holding `values`, retaining ids in
    /// order. Errors, building nothing, if a component is NaN.
    pub fn vec_3_float(values: Vec<[f64; 3]>) -> Result<Self> {
        let (value_ids, values) = checked_columns(values, floats_in_domain)?;

        Ok(Self {
            value_ids,
            kind: VoxValuePoolKind::Vec3Float(values),
        })
    }

    /// Creates a `vec-3-int` value pool holding `values`, retaining ids in
    /// order. Errors, building nothing, if a component's magnitude exceeds
    /// `2^53 - 1`.
    pub fn vec_3_int(values: Vec<[i64; 3]>) -> Result<Self> {
        let (value_ids, values) = checked_columns(values, ints_in_domain)?;

        Ok(Self {
            value_ids,
            kind: VoxValuePoolKind::Vec3Int(values),
        })
    }

    /// Creates a `vec-4-float` value pool holding `values`, retaining ids in
    /// order. Errors, building nothing, if a component is NaN.
    pub fn vec_4_float(values: Vec<[f64; 4]>) -> Result<Self> {
        let (value_ids, values) = checked_columns(values, floats_in_domain)?;

        Ok(Self {
            value_ids,
            kind: VoxValuePoolKind::Vec4Float(values),
        })
    }

    /// Creates a `vec-4-int` value pool holding `values`, retaining ids in
    /// order. Errors, building nothing, if a component's magnitude exceeds
    /// `2^53 - 1`.
    pub fn vec_4_int(values: Vec<[i64; 4]>) -> Result<Self> {
        let (value_ids, values) = checked_columns(values, ints_in_domain)?;

        Ok(Self {
            value_ids,
            kind: VoxValuePoolKind::Vec4Int(values),
        })
    }

    /// The kind, for matching.
    pub fn kind(&self) -> &VoxValuePoolKind {
        &self.kind
    }

    /// Appends a `bool` value and returns its id. Errors, changing nothing, if
    /// the value pool holds another kind.
    pub fn retain_boolean_value(&mut self, value: bool) -> Result<U32Id<BVoxValuePoolValue>> {
        let VoxValuePoolKind::Bool(values) = &mut self.kind else {
            return Err(Error::RetainedValueKind);
        };

        Ok(retain_into(&mut self.value_ids, values, value))
    }

    /// Appends a `float` value and returns its id. Errors, changing nothing, if
    /// the value pool holds another kind or `value` is NaN.
    pub fn retain_float_value(&mut self, value: f64) -> Result<U32Id<BVoxValuePoolValue>> {
        let VoxValuePoolKind::Float(values) = &mut self.kind else {
            return Err(Error::RetainedValueKind);
        };

        if !float_in_domain(&value) {
            return Err(Error::MalformedRetainedValue);
        }

        Ok(retain_into(&mut self.value_ids, values, value))
    }

    /// Appends an `int` value and returns its id. Errors, changing nothing, if
    /// the value pool holds another kind or `value`'s magnitude exceeds
    /// `2^53 - 1`.
    pub fn retain_int_value(&mut self, value: i64) -> Result<U32Id<BVoxValuePoolValue>> {
        let VoxValuePoolKind::Int(values) = &mut self.kind else {
            return Err(Error::RetainedValueKind);
        };

        if !int_in_domain(&value) {
            return Err(Error::MalformedRetainedValue);
        }

        Ok(retain_into(&mut self.value_ids, values, value))
    }

    /// Appends a `json` value and returns its id. Errors, changing nothing, if
    /// the value pool holds another kind.
    pub fn retain_json_value(&mut self, value: VoxValue) -> Result<U32Id<BVoxValuePoolValue>> {
        let VoxValuePoolKind::Json(values) = &mut self.kind else {
            return Err(Error::RetainedValueKind);
        };

        Ok(retain_into(&mut self.value_ids, values, value))
    }

    /// Appends a `string` value and returns its id. Errors, changing nothing,
    /// if the value pool holds another kind.
    pub fn retain_string_value(&mut self, value: String) -> Result<U32Id<BVoxValuePoolValue>> {
        let VoxValuePoolKind::String(values) = &mut self.kind else {
            return Err(Error::RetainedValueKind);
        };

        Ok(retain_into(&mut self.value_ids, values, value))
    }

    /// Appends a `vec-2-float` value and returns its id. Errors, changing
    /// nothing, if the value pool holds another kind or a component is NaN.
    pub fn retain_vec_2_float_value(
        &mut self,
        value: [f64; 2],
    ) -> Result<U32Id<BVoxValuePoolValue>> {
        let VoxValuePoolKind::Vec2Float(values) = &mut self.kind else {
            return Err(Error::RetainedValueKind);
        };

        if !floats_in_domain(&value) {
            return Err(Error::MalformedRetainedValue);
        }

        Ok(retain_into(&mut self.value_ids, values, value))
    }

    /// Appends a `vec-2-int` value and returns its id. Errors, changing
    /// nothing, if the value pool holds another kind or a component's magnitude
    /// exceeds `2^53 - 1`.
    pub fn retain_vec_2_int_value(&mut self, value: [i64; 2]) -> Result<U32Id<BVoxValuePoolValue>> {
        let VoxValuePoolKind::Vec2Int(values) = &mut self.kind else {
            return Err(Error::RetainedValueKind);
        };

        if !ints_in_domain(&value) {
            return Err(Error::MalformedRetainedValue);
        }

        Ok(retain_into(&mut self.value_ids, values, value))
    }

    /// Appends a `vec-3-float` value and returns its id. Errors, changing
    /// nothing, if the value pool holds another kind or a component is NaN.
    pub fn retain_vec_3_float_value(
        &mut self,
        value: [f64; 3],
    ) -> Result<U32Id<BVoxValuePoolValue>> {
        let VoxValuePoolKind::Vec3Float(values) = &mut self.kind else {
            return Err(Error::RetainedValueKind);
        };

        if !floats_in_domain(&value) {
            return Err(Error::MalformedRetainedValue);
        }

        Ok(retain_into(&mut self.value_ids, values, value))
    }

    /// Appends a `vec-3-int` value and returns its id. Errors, changing
    /// nothing, if the value pool holds another kind or a component's magnitude
    /// exceeds `2^53 - 1`.
    pub fn retain_vec_3_int_value(&mut self, value: [i64; 3]) -> Result<U32Id<BVoxValuePoolValue>> {
        let VoxValuePoolKind::Vec3Int(values) = &mut self.kind else {
            return Err(Error::RetainedValueKind);
        };

        if !ints_in_domain(&value) {
            return Err(Error::MalformedRetainedValue);
        }

        Ok(retain_into(&mut self.value_ids, values, value))
    }

    /// Appends a `vec-4-float` value and returns its id. Errors, changing
    /// nothing, if the value pool holds another kind or a component is NaN.
    pub fn retain_vec_4_float_value(
        &mut self,
        value: [f64; 4],
    ) -> Result<U32Id<BVoxValuePoolValue>> {
        let VoxValuePoolKind::Vec4Float(values) = &mut self.kind else {
            return Err(Error::RetainedValueKind);
        };

        if !floats_in_domain(&value) {
            return Err(Error::MalformedRetainedValue);
        }

        Ok(retain_into(&mut self.value_ids, values, value))
    }

    /// Appends a `vec-4-int` value and returns its id. Errors, changing
    /// nothing, if the value pool holds another kind or a component's magnitude
    /// exceeds `2^53 - 1`.
    pub fn retain_vec_4_int_value(&mut self, value: [i64; 4]) -> Result<U32Id<BVoxValuePoolValue>> {
        let VoxValuePoolKind::Vec4Int(values) = &mut self.kind else {
            return Err(Error::RetainedValueKind);
        };

        if !ints_in_domain(&value) {
            return Err(Error::MalformedRetainedValue);
        }

        Ok(retain_into(&mut self.value_ids, values, value))
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

    /// The id of the first value outside its kind's value domain, or `None` if
    /// every value is within it. The checked constructors and the
    /// `retain_*_value` methods gate on the domain, so a flaw found later is a
    /// voxcore bug. [`VoxMain::validate`](crate::VoxMain::validate) audits for
    /// one anyway.
    pub(crate) fn first_out_of_domain_value(&self) -> Option<U32Id<BVoxValuePoolValue>> {
        let ids = &self.value_ids;

        match &self.kind {
            VoxValuePoolKind::Float(values) => first_out_of_domain(ids, values, float_in_domain),

            VoxValuePoolKind::Int(values) => first_out_of_domain(ids, values, int_in_domain),

            VoxValuePoolKind::Vec2Float(values) => {
                first_out_of_domain(ids, values, floats_in_domain)
            }

            VoxValuePoolKind::Vec2Int(values) => first_out_of_domain(ids, values, ints_in_domain),

            VoxValuePoolKind::Vec3Float(values) => {
                first_out_of_domain(ids, values, floats_in_domain)
            }

            VoxValuePoolKind::Vec3Int(values) => first_out_of_domain(ids, values, ints_in_domain),

            VoxValuePoolKind::Vec4Float(values) => {
                first_out_of_domain(ids, values, floats_in_domain)
            }

            VoxValuePoolKind::Vec4Int(values) => first_out_of_domain(ids, values, ints_in_domain),

            VoxValuePoolKind::Bool(_) | VoxValuePoolKind::Json(_) | VoxValuePoolKind::String(_) => {
                None
            }
        }
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

    /// Number of values.
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

impl Clone for VoxValuePool {
    fn clone(&self) -> Self {
        let ids = &self.value_ids;

        // Safety: the column holds a value for every id in the id pool.
        let kind = unsafe {
            match &self.kind {
                VoxValuePoolKind::Bool(values) => {
                    VoxValuePoolKind::Bool(values.clone_retained(ids))
                }

                VoxValuePoolKind::Float(values) => {
                    VoxValuePoolKind::Float(values.clone_retained(ids))
                }

                VoxValuePoolKind::Int(values) => VoxValuePoolKind::Int(values.clone_retained(ids)),

                VoxValuePoolKind::Json(values) => {
                    VoxValuePoolKind::Json(values.clone_retained(ids))
                }

                VoxValuePoolKind::String(values) => {
                    VoxValuePoolKind::String(values.clone_retained(ids))
                }

                VoxValuePoolKind::Vec2Float(values) => {
                    VoxValuePoolKind::Vec2Float(values.clone_retained(ids))
                }

                VoxValuePoolKind::Vec2Int(values) => {
                    VoxValuePoolKind::Vec2Int(values.clone_retained(ids))
                }

                VoxValuePoolKind::Vec3Float(values) => {
                    VoxValuePoolKind::Vec3Float(values.clone_retained(ids))
                }

                VoxValuePoolKind::Vec3Int(values) => {
                    VoxValuePoolKind::Vec3Int(values.clone_retained(ids))
                }

                VoxValuePoolKind::Vec4Float(values) => {
                    VoxValuePoolKind::Vec4Float(values.clone_retained(ids))
                }

                VoxValuePoolKind::Vec4Int(values) => {
                    VoxValuePoolKind::Vec4Int(values.clone_retained(ids))
                }
            }
        };

        Self {
            value_ids: ids.clone(),
            kind,
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

/// Whether `value` is within the float value domain.
fn float_in_domain(value: &f64) -> bool {
    !value.is_nan()
}

/// Whether every component of a float vector is within the float value domain.
fn floats_in_domain<const N: usize>(components: &[f64; N]) -> bool {
    components.iter().all(float_in_domain)
}

/// Whether `value` is within the int value domain.
fn int_in_domain(value: &i64) -> bool {
    (-MAX_INT_MAGNITUDE..=MAX_INT_MAGNITUDE).contains(value)
}

/// Whether every component of an int vector is within the int value domain.
fn ints_in_domain<const N: usize>(components: &[i64; N]) -> bool {
    components.iter().all(int_in_domain)
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

/// Builds the columns for `values` as [`columns`] does. Errors, building
/// nothing, on the first value outside the domain. Ids retain in order, so a
/// value's index is the id it would take.
fn checked_columns<T>(
    values: Vec<T>,
    in_domain: fn(&T) -> bool,
) -> Result<(IdStruct<BVoxValuePoolValue>, IdField<BVoxValuePoolValue, T>)> {
    for (index, value) in (0..).zip(&values) {
        if !in_domain(value) {
            return Err(Error::MalformedValuePoolValue {
                value_id: U32Id::from_u32(index),
            });
        }
    }

    Ok(columns(values))
}

/// The id of the first value in `column` outside the domain.
fn first_out_of_domain<T>(
    ids: &IdStruct<BVoxValuePoolValue>,
    column: &IdField<BVoxValuePoolValue, T>,
    in_domain: fn(&T) -> bool,
) -> Option<U32Id<BVoxValuePoolValue>> {
    // Safety: the column holds a value for every id in its id pool.
    let values = unsafe { column.iter(ids) };

    for (value_id, value) in ids.iter().zip(values) {
        if !in_domain(value) {
            return Some(value_id);
        }
    }

    None
}

/// Retains an id in `ids` and writes `value` to it in `column`.
fn retain_into<T>(
    ids: &mut IdStruct<BVoxValuePoolValue>,
    column: &mut IdField<BVoxValuePoolValue, T>,
    value: T,
) -> U32Id<BVoxValuePoolValue> {
    let value_id = ids.retain();
    column.retain(value_id, value);
    value_id
}

#[cfg(test)]
mod tests {
    use crate::{
        BVoxValuePoolValue, Error, VoxValue, VoxValuePool, VoxValuePoolKind, VoxValuePoolValueRef,
        vox_value_pool::columns,
    };
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

    #[test]
    fn retain_int_value_appends_to_the_listing() {
        let mut value_pool = VoxValuePool::int(vec![10, 20, 30]).unwrap();
        value_pool.release_value_stable(value_id(0));

        // The released id comes back, listed last.
        assert_eq!(value_pool.retain_int_value(40), Ok(value_id(0)));
        assert_eq!(value_pool.retain_int_value(50), Ok(value_id(3)));
        assert_eq!(value_pool, VoxValuePool::int(vec![20, 30, 40, 50]).unwrap());
    }

    #[test]
    fn each_retain_appends_its_kind() {
        let mut booleans = VoxValuePool::boolean(vec![]);
        booleans.retain_boolean_value(true).unwrap();
        assert_eq!(booleans, VoxValuePool::boolean(vec![true]));

        let mut floats = VoxValuePool::float(vec![]).unwrap();
        floats.retain_float_value(f64::INFINITY).unwrap();
        assert_eq!(floats, VoxValuePool::float(vec![f64::INFINITY]).unwrap());

        let mut ints = VoxValuePool::int(vec![]).unwrap();
        ints.retain_int_value(-1).unwrap();
        assert_eq!(ints, VoxValuePool::int(vec![-1]).unwrap());

        let mut jsons = VoxValuePool::json(vec![]);
        jsons.retain_json_value(VoxValue::Null).unwrap();
        assert_eq!(jsons, VoxValuePool::json(vec![VoxValue::Null]));

        let mut strings = VoxValuePool::string(vec![]);
        strings.retain_string_value("rust".to_owned()).unwrap();
        assert_eq!(strings, VoxValuePool::string(vec!["rust".to_owned()]));

        let mut vec_2_floats = VoxValuePool::vec_2_float(vec![]).unwrap();
        vec_2_floats.retain_vec_2_float_value([0.0, 1.0]).unwrap();
        assert_eq!(
            vec_2_floats,
            VoxValuePool::vec_2_float(vec![[0.0, 1.0]]).unwrap()
        );

        let mut vec_2_ints = VoxValuePool::vec_2_int(vec![]).unwrap();
        vec_2_ints.retain_vec_2_int_value([0, 1]).unwrap();
        assert_eq!(vec_2_ints, VoxValuePool::vec_2_int(vec![[0, 1]]).unwrap());

        let mut vec_3_floats = VoxValuePool::vec_3_float(vec![]).unwrap();
        vec_3_floats
            .retain_vec_3_float_value([0.0, 1.0, 2.0])
            .unwrap();
        assert_eq!(
            vec_3_floats,
            VoxValuePool::vec_3_float(vec![[0.0, 1.0, 2.0]]).unwrap()
        );

        let mut vec_3_ints = VoxValuePool::vec_3_int(vec![]).unwrap();
        vec_3_ints.retain_vec_3_int_value([0, 1, 2]).unwrap();
        assert_eq!(
            vec_3_ints,
            VoxValuePool::vec_3_int(vec![[0, 1, 2]]).unwrap()
        );

        let mut vec_4_floats = VoxValuePool::vec_4_float(vec![]).unwrap();
        vec_4_floats
            .retain_vec_4_float_value([0.0, 1.0, 2.0, 3.0])
            .unwrap();
        assert_eq!(
            vec_4_floats,
            VoxValuePool::vec_4_float(vec![[0.0, 1.0, 2.0, 3.0]]).unwrap()
        );

        let mut vec_4_ints = VoxValuePool::vec_4_int(vec![]).unwrap();
        vec_4_ints.retain_vec_4_int_value([0, 1, 2, 3]).unwrap();
        assert_eq!(
            vec_4_ints,
            VoxValuePool::vec_4_int(vec![[0, 1, 2, 3]]).unwrap()
        );
    }

    #[test]
    fn a_retain_rejects_another_kind_without_spending_an_id() {
        let mut value_pool = VoxValuePool::float(vec![0.5]).unwrap();

        assert_eq!(
            value_pool.retain_int_value(1),
            Err(Error::RetainedValueKind)
        );
        assert_eq!(
            value_pool.retain_float_value(f64::NAN),
            Err(Error::MalformedRetainedValue)
        );
        assert_eq!(value_pool, VoxValuePool::float(vec![0.5]).unwrap());
        assert_eq!(value_pool.retain_float_value(1.0), Ok(value_id(1)));
    }

    #[test]
    fn each_numeric_retain_rejects_an_out_of_domain_value() {
        const PAST_CAP: i64 = 1 << 53;
        let malformed = Err(Error::MalformedRetainedValue);

        let mut floats = VoxValuePool::float(vec![]).unwrap();
        assert_eq!(floats.retain_float_value(f64::NAN), malformed);

        let mut ints = VoxValuePool::int(vec![]).unwrap();
        assert_eq!(ints.retain_int_value(-PAST_CAP), malformed);

        let mut vec_2_floats = VoxValuePool::vec_2_float(vec![]).unwrap();
        assert_eq!(
            vec_2_floats.retain_vec_2_float_value([0.0, f64::NAN]),
            malformed
        );

        let mut vec_2_ints = VoxValuePool::vec_2_int(vec![]).unwrap();
        assert_eq!(vec_2_ints.retain_vec_2_int_value([0, PAST_CAP]), malformed);

        let mut vec_3_floats = VoxValuePool::vec_3_float(vec![]).unwrap();
        assert_eq!(
            vec_3_floats.retain_vec_3_float_value([0.0, 0.0, f64::NAN]),
            malformed
        );

        let mut vec_3_ints = VoxValuePool::vec_3_int(vec![]).unwrap();
        assert_eq!(
            vec_3_ints.retain_vec_3_int_value([0, 0, PAST_CAP]),
            malformed
        );

        let mut vec_4_floats = VoxValuePool::vec_4_float(vec![]).unwrap();
        assert_eq!(
            vec_4_floats.retain_vec_4_float_value([0.0, 0.0, 0.0, f64::NAN]),
            malformed
        );

        let mut vec_4_ints = VoxValuePool::vec_4_int(vec![]).unwrap();
        assert_eq!(
            vec_4_ints.retain_vec_4_int_value([0, 0, 0, PAST_CAP]),
            malformed
        );
    }

    #[test]
    fn the_audit_finds_a_value_past_the_gates() {
        // A struct literal skips the constructor's gate, as a voxcore bug would.
        let (value_ids, values) = columns(vec![[0.0, 0.0], [0.0, 1.0], [f64::NAN, 0.0]]);
        let mut value_pool = VoxValuePool {
            value_ids,
            kind: VoxValuePoolKind::Vec2Float(values),
        };

        assert_eq!(value_pool.first_out_of_domain_value(), Some(value_id(2)));

        // Listed first, the bad value still reports its own id.
        value_pool.move_value(value_id(2), 0).unwrap();
        assert_eq!(value_pool.first_out_of_domain_value(), Some(value_id(2)));

        value_pool.release_value_stable(value_id(2));
        assert_eq!(value_pool.first_out_of_domain_value(), None);
    }

    #[test]
    fn a_clone_keeps_ids_and_holes() {
        let mut value_pool =
            VoxValuePool::string(vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]);

        value_pool.release_value_stable(value_id(1));

        let copy = value_pool.clone();

        assert_eq!(copy, value_pool);
        assert_eq!(
            copy.iter_values().collect::<Vec<_>>(),
            [
                (value_id(0), VoxValuePoolValueRef::String("a")),
                (value_id(2), VoxValuePoolValueRef::String("c")),
            ]
        );
    }
}
