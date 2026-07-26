use crate::{BVoxPoolValue, VoxBound, VoxPoolValueRef, VoxValue, VoxValuePoolKind};
use branded_id::{
    U32Id,
    soa::{IdField, IdRemap, IdStruct},
};
use std::mem;

/// A shared pool of values, one [`VoxValuePoolKind`] column keyed by a pool of
/// value ids.
///
/// Build a pool with the constructor for its kind (for example
/// [`float`](Self::float) or [`srgba`](Self::srgba)), which retains one id per
/// value in the given order. Read values back by id with
/// [`value`](Self::value) or in listing order with
/// [`iter_values`](Self::iter_values), and match [`kind`](Self::kind) for the
/// kind and bounds. [`VoxMain::validate`](crate::VoxMain::validate) checks
/// each value against its kind and the bounds.
#[derive(Debug)]
pub struct VoxValuePool {
    /// Value id pool. Its listing order is the pool's value order.
    value_ids: IdStruct<BVoxPoolValue>,

    /// The kind and its typed value column, keyed by `value_ids`.
    kind: VoxValuePoolKind,
}

impl VoxValuePool {
    /// Creates a `json` pool holding `values`, retaining ids in order.
    pub fn json(values: Vec<VoxValue>) -> Self {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Json { values },
        }
    }

    /// Creates a `bool` pool holding `values`, retaining ids in order.
    pub fn boolean(values: Vec<bool>) -> Self {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Bool { values },
        }
    }

    /// Creates a `float` pool bounded by `min`/`max` holding `values`,
    /// retaining ids in order.
    pub fn float(min: VoxBound, max: VoxBound, values: Vec<f64>) -> Self {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Float { min, max, values },
        }
    }

    /// Creates an `int` pool bounded by `min`/`max` holding `values`,
    /// retaining ids in order.
    pub fn int(min: VoxBound, max: VoxBound, values: Vec<i64>) -> Self {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Int { min, max, values },
        }
    }

    /// Creates a `string` pool holding `values`, retaining ids in order.
    pub fn string(values: Vec<String>) -> Self {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::String { values },
        }
    }

    /// Creates an `srgb` pool holding `values`, retaining ids in order.
    pub fn srgb(values: Vec<[f64; 3]>) -> Self {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Srgb { values },
        }
    }

    /// Creates an `srgba` pool holding `values`, retaining ids in order.
    pub fn srgba(values: Vec<[f64; 4]>) -> Self {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Srgba { values },
        }
    }

    /// Creates a `linear-rgb` pool holding `values`, retaining ids in order.
    pub fn linear_rgb(values: Vec<[f64; 3]>) -> Self {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::LinearRgb { values },
        }
    }

    /// Creates a `linear-rgba` pool holding `values`, retaining ids in order.
    pub fn linear_rgba(values: Vec<[f64; 4]>) -> Self {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::LinearRgba { values },
        }
    }

    /// The kind and bounds, for matching. Read the values through
    /// [`value`](Self::value) and [`iter_values`](Self::iter_values).
    pub fn kind(&self) -> &VoxValuePoolKind {
        &self.kind
    }

    /// The number of values in the pool, across every kind.
    pub fn values_len(&self) -> usize {
        self.value_ids.len()
    }

    /// Whether `id` is one of this pool's values.
    pub fn contains_value(&self, id: U32Id<BVoxPoolValue>) -> bool {
        self.value_ids.is_retained(id)
    }

    /// The value at `id`, typed by the pool's kind, or `None` if `id` is not
    /// one of this pool's values.
    pub fn value(&self, id: U32Id<BVoxPoolValue>) -> Option<VoxPoolValueRef<'_>> {
        self.value_ids.is_retained(id).then(|| self.value_ref(id))
    }

    /// Values in listing order, as `(id, value)`.
    pub fn iter_values(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxPoolValue>, VoxPoolValueRef<'_>)> + '_ {
        self.value_ids
            .iter()
            .map(move |id| (id, self.value_ref(id)))
    }

    /// The listing position of value `id`, or `None` if `id` is not one of
    /// this pool's values.
    pub fn value_index(&self, id: U32Id<BVoxPoolValue>) -> Option<usize> {
        self.value_ids.index_of(id)
    }

    /// Moves value `id` to listing position `index`, shifting the values
    /// between its old and new positions one slot. Returns `None` and changes
    /// nothing if `id` is not one of this pool's values or `index` is at or
    /// past [`values_len`](Self::values_len).
    pub fn move_value(&mut self, id: U32Id<BVoxPoolValue>, index: usize) -> Option<()> {
        self.value_ids.try_move_to(id, index)
    }

    /// Deep copy. Liveness lives in the id pool, so the column can't derive
    /// `Clone`. Rebuild it against the cloned pool.
    pub fn clone_pool(&self) -> Self {
        let kind = match &self.kind {
            VoxValuePoolKind::Json { values } => VoxValuePoolKind::Json {
                values: cloned(&self.value_ids, values),
            },
            VoxValuePoolKind::Bool { values } => VoxValuePoolKind::Bool {
                values: cloned(&self.value_ids, values),
            },
            VoxValuePoolKind::Float { min, max, values } => VoxValuePoolKind::Float {
                min: *min,
                max: *max,
                values: cloned(&self.value_ids, values),
            },
            VoxValuePoolKind::Int { min, max, values } => VoxValuePoolKind::Int {
                min: *min,
                max: *max,
                values: cloned(&self.value_ids, values),
            },
            VoxValuePoolKind::String { values } => VoxValuePoolKind::String {
                values: cloned(&self.value_ids, values),
            },
            VoxValuePoolKind::Srgb { values } => VoxValuePoolKind::Srgb {
                values: cloned(&self.value_ids, values),
            },
            VoxValuePoolKind::Srgba { values } => VoxValuePoolKind::Srgba {
                values: cloned(&self.value_ids, values),
            },
            VoxValuePoolKind::LinearRgb { values } => VoxValuePoolKind::LinearRgb {
                values: cloned(&self.value_ids, values),
            },
            VoxValuePoolKind::LinearRgba { values } => VoxValuePoolKind::LinearRgba {
                values: cloned(&self.value_ids, values),
            },
        };

        Self {
            value_ids: self.value_ids.clone(),
            kind,
        }
    }

    /// Releases value `id`, keeping the surviving values' listing order. The
    /// id must be one of this pool's values. The caller repoints any palette
    /// cell drawing it first.
    pub(crate) fn release_value_stable(&mut self, id: U32Id<BVoxPoolValue>) {
        // Safety: the id is retained, so it has a value in the column.
        unsafe {
            match &mut self.kind {
                VoxValuePoolKind::Json { values } => values.release(id),
                VoxValuePoolKind::Bool { values } => values.release(id),
                VoxValuePoolKind::Float { values, .. } => values.release(id),
                VoxValuePoolKind::Int { values, .. } => values.release(id),
                VoxValuePoolKind::String { values } => values.release(id),
                VoxValuePoolKind::Srgb { values } => values.release(id),
                VoxValuePoolKind::Srgba { values } => values.release(id),
                VoxValuePoolKind::LinearRgb { values } => values.release(id),
                VoxValuePoolKind::LinearRgba { values } => values.release(id),
            }
        }
        self.value_ids.release_stable(id);
    }

    /// Rewrites the listing order to `new_order`. `None`, changing nothing, if
    /// `new_order` does not list every value id exactly once.
    pub(crate) fn set_value_order(&mut self, new_order: &[U32Id<BVoxPoolValue>]) -> Option<()> {
        self.value_ids.try_set_order(new_order)
    }

    /// Compacts the value id pool back to a contiguous `0..len` in listing
    /// order and returns the relabeling, so the caller can translate the
    /// palette cells that point at these values.
    pub(crate) fn gc_values(&mut self) -> IdRemap<BVoxPoolValue, u32> {
        let remap = self.value_ids.gc();
        // Safety: the column was in sync with the pre-gc pool, and nothing has
        // retained or released since.
        unsafe {
            match &mut self.kind {
                VoxValuePoolKind::Json { values } => values.gc(&remap),
                VoxValuePoolKind::Bool { values } => values.gc(&remap),
                VoxValuePoolKind::Float { values, .. } => values.gc(&remap),
                VoxValuePoolKind::Int { values, .. } => values.gc(&remap),
                VoxValuePoolKind::String { values } => values.gc(&remap),
                VoxValuePoolKind::Srgb { values } => values.gc(&remap),
                VoxValuePoolKind::Srgba { values } => values.gc(&remap),
                VoxValuePoolKind::LinearRgb { values } => values.gc(&remap),
                VoxValuePoolKind::LinearRgba { values } => values.gc(&remap),
            }
        }
        remap
    }

    /// The typed ref for a retained `id`.
    fn value_ref(&self, id: U32Id<BVoxPoolValue>) -> VoxPoolValueRef<'_> {
        // Safety: the id is retained, so it has a value in the column.
        unsafe {
            match &self.kind {
                VoxValuePoolKind::Json { values } => VoxPoolValueRef::Json(values.get(id)),
                VoxValuePoolKind::Bool { values } => VoxPoolValueRef::Bool(*values.get(id)),
                VoxValuePoolKind::Float { values, .. } => VoxPoolValueRef::Float(*values.get(id)),
                VoxValuePoolKind::Int { values, .. } => VoxPoolValueRef::Int(*values.get(id)),
                VoxValuePoolKind::String { values } => VoxPoolValueRef::String(values.get(id)),
                VoxValuePoolKind::Srgb { values } => VoxPoolValueRef::Srgb(values.get(id)),
                VoxValuePoolKind::Srgba { values } => VoxPoolValueRef::Srgba(values.get(id)),
                VoxValuePoolKind::LinearRgb { values } => {
                    VoxPoolValueRef::LinearRgb(values.get(id))
                }
                VoxValuePoolKind::LinearRgba { values } => {
                    VoxPoolValueRef::LinearRgba(values.get(id))
                }
            }
        }
    }
}

impl Drop for VoxValuePool {
    fn drop(&mut self) {
        // Safety: the column holds a value for every id in the pool.
        unsafe {
            match &mut self.kind {
                VoxValuePoolKind::Json { values } => values.release_all(&self.value_ids),
                VoxValuePoolKind::Bool { values } => values.release_all(&self.value_ids),
                VoxValuePoolKind::Float { values, .. } => values.release_all(&self.value_ids),
                VoxValuePoolKind::Int { values, .. } => values.release_all(&self.value_ids),
                VoxValuePoolKind::String { values } => values.release_all(&self.value_ids),
                VoxValuePoolKind::Srgb { values } => values.release_all(&self.value_ids),
                VoxValuePoolKind::Srgba { values } => values.release_all(&self.value_ids),
                VoxValuePoolKind::LinearRgb { values } => values.release_all(&self.value_ids),
                VoxValuePoolKind::LinearRgba { values } => values.release_all(&self.value_ids),
            }
        }
    }
}

impl PartialEq for VoxValuePool {
    /// Compares kind, bounds, and values in listing order. The id labels
    /// underneath the listing do not take part.
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(&self.kind) == mem::discriminant(&other.kind)
            && bounds(&self.kind) == bounds(&other.kind)
            && self
                .iter_values()
                .map(|(_, value)| value)
                .eq(other.iter_values().map(|(_, value)| value))
    }
}

/// A bounded kind's `min` and `max`, or `None` for a kind carrying no bounds.
fn bounds(kind: &VoxValuePoolKind) -> Option<(VoxBound, VoxBound)> {
    match kind {
        VoxValuePoolKind::Float { min, max, .. } | VoxValuePoolKind::Int { min, max, .. } => {
            Some((*min, *max))
        }
        _ => None,
    }
}

/// Builds the paired id pool and value column for `values`, retaining ids in
/// order.
fn columns<T>(values: Vec<T>) -> (IdStruct<BVoxPoolValue>, IdField<BVoxPoolValue, T>) {
    let mut ids = IdStruct::new();
    let mut column = IdField::with_capacity(values.len());
    for value in values {
        let id = ids.retain();
        column.retain(id, value);
    }
    (ids, column)
}

/// Clones the values retained in `column` against the same ids.
fn cloned<T: Clone>(
    ids: &IdStruct<BVoxPoolValue>,
    column: &IdField<BVoxPoolValue, T>,
) -> IdField<BVoxPoolValue, T> {
    let mut copy = IdField::with_capacity(ids.len());
    for id in ids.iter() {
        // Safety: retained ids have a value.
        copy.retain(id, unsafe { column.get(id) }.clone());
    }
    copy
}

#[cfg(test)]
mod tests {
    use crate::{VoxBound, VoxPoolValueRef, VoxValuePool};
    use branded_id::U32Id;

    #[test]
    fn bounded_float_pool_reads_back_in_order() {
        let pool = VoxValuePool::float(VoxBound::Number(0.0), VoxBound::None, vec![0.0, 0.5, 1.0]);

        assert_eq!(pool.values_len(), 3);
        let values: Vec<_> = pool.iter_values().collect();
        assert_eq!(values.len(), 3);
        assert_eq!(values[1], (U32Id::from_u32(1), VoxPoolValueRef::Float(0.5)));
        assert_eq!(
            pool.value(U32Id::from_u32(2)),
            Some(VoxPoolValueRef::Float(1.0))
        );
        assert_eq!(pool.value(U32Id::from_u32(3)), None);
    }

    #[test]
    fn color_pool_holds_typed_float_components() {
        let pool = VoxValuePool::srgba(vec![[1.0, 0.0, 0.0, 1.0]]);

        assert_eq!(pool.values_len(), 1);
        assert_eq!(
            pool.value(U32Id::from_u32(0)),
            Some(VoxPoolValueRef::Srgba(&[1.0, 0.0, 0.0, 1.0]))
        );
    }

    #[test]
    fn pools_compare_by_kind_bounds_and_ordered_values() {
        let a = VoxValuePool::int(VoxBound::None, VoxBound::None, vec![1, 2]);
        let mut b = VoxValuePool::int(VoxBound::None, VoxBound::None, vec![2, 1]);
        assert_ne!(a, b);

        // Moving b's values into a's order makes the pools equal, even though
        // their id labels now differ per position.
        b.move_value(U32Id::from_u32(1), 0).unwrap();
        assert_eq!(a, b);

        // Same values, different bounds.
        let c = VoxValuePool::int(VoxBound::Number(0.0), VoxBound::None, vec![1, 2]);
        assert_ne!(a, c);

        // Same shape, different kind.
        let d = VoxValuePool::srgb(vec![[0.0, 0.0, 0.0]]);
        let e = VoxValuePool::linear_rgb(vec![[0.0, 0.0, 0.0]]);
        assert_ne!(d, e);
    }

    #[test]
    fn move_value_reorders_the_listing_and_validates() {
        let mut pool = VoxValuePool::string(vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]);
        let b = U32Id::from_u32(1);

        assert_eq!(pool.move_value(b, 2), Some(()));
        let order: Vec<_> = pool
            .iter_values()
            .map(|(_, value)| match value {
                VoxPoolValueRef::String(text) => text.to_owned(),
                other => panic!("unexpected value {other:?}"),
            })
            .collect();
        assert_eq!(order, ["a", "c", "b"]);
        assert_eq!(pool.value_index(b), Some(2));

        // An out-of-range index and an unknown id are rejected.
        assert_eq!(pool.move_value(b, 3), None);
        assert_eq!(pool.move_value(U32Id::from_u32(9), 0), None);
    }

    #[test]
    fn clone_pool_is_an_independent_deep_copy() {
        let mut pool = VoxValuePool::string(vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]);
        // Hole the pool first: equality compares values in listing order, so
        // only reading back by id catches a clone that relabels.
        pool.release_value_stable(U32Id::from_u32(1));

        let mut copy = pool.clone_pool();
        assert_eq!(pool, copy);
        assert_eq!(copy.value(U32Id::from_u32(1)), None);
        assert_eq!(
            copy.value(U32Id::from_u32(2)),
            Some(VoxPoolValueRef::String("c"))
        );

        copy.move_value(U32Id::from_u32(0), 1).unwrap();
        assert_ne!(pool, copy);
    }
}
