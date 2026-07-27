use crate::{
    BVoxPoolValue, Error, Result, VoxBound, VoxPoolFlaw, VoxPoolValueRef, VoxValue,
    VoxValuePoolKind,
};
use branded_id::{
    U32Id,
    soa::{IdField, IdRemap, IdStruct},
};
use std::mem;

/// A shared pool of values, one [`VoxValuePoolKind`] column keyed by a pool of
/// value ids.
///
/// Build a pool with the constructor for its kind (for example
/// [`float`](Self::float) or [`srgba`](Self::srgba)), which checks its input
/// and retains one id per value in the given order. Read values back by id with
/// [`value`](Self::value) or in listing order with
/// [`iter_values`](Self::iter_values), and match [`kind`](Self::kind) for the
/// kind and bounds.
#[derive(Debug)]
pub struct VoxValuePool {
    /// Value id pool. Its listing order is the pool's value order.
    value_ids: IdStruct<BVoxPoolValue>,

    /// The kind and its typed value column, keyed by `value_ids`.
    kind: VoxValuePoolKind,
}

impl VoxValuePool {
    /// Creates a `json` pool holding `values`, retaining ids in order.
    /// Errors, building nothing, if `values` is empty.
    pub fn json(values: Vec<VoxValue>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Json { values },
        }
        .checked()
    }

    /// Creates a `bool` pool holding `values`, retaining ids in order.
    /// Errors, building nothing, if `values` is empty.
    pub fn boolean(values: Vec<bool>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Bool { values },
        }
        .checked()
    }

    /// Creates a `float` pool bounded by `min`/`max` holding `values`,
    /// retaining ids in order. Errors, building nothing, if:
    ///
    /// 1. `values` is empty
    /// 2. a bound is non-finite or the bounds are unordered
    /// 3. a value is non-finite or outside the bounds
    pub fn float(min: VoxBound, max: VoxBound, values: Vec<f64>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Float { min, max, values },
        }
        .checked()
    }

    /// Creates an `int` pool bounded by `min`/`max` holding `values`,
    /// retaining ids in order. Errors, building nothing, if:
    ///
    /// 1. `values` is empty
    /// 2. a bound is non-finite or not integer-valued, or the bounds are
    ///    unordered
    /// 3. a value is outside the bounds
    pub fn int(min: VoxBound, max: VoxBound, values: Vec<i64>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Int { min, max, values },
        }
        .checked()
    }

    /// Creates a `string` pool holding `values`, retaining ids in order.
    /// Errors, building nothing, if `values` is empty.
    pub fn string(values: Vec<String>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::String { values },
        }
        .checked()
    }

    /// Creates an `srgb` pool holding `values`, retaining ids in order.
    /// Errors, building nothing, if `values` is empty or a component is
    /// outside `[0, 1]`.
    pub fn srgb(values: Vec<[f64; 3]>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Srgb { values },
        }
        .checked()
    }

    /// Creates an `srgba` pool holding `values`, retaining ids in order.
    /// Errors, building nothing, if `values` is empty or a component is
    /// outside `[0, 1]`.
    pub fn srgba(values: Vec<[f64; 4]>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::Srgba { values },
        }
        .checked()
    }

    /// Creates a `linear-rgb` pool holding `values`, retaining ids in order.
    /// Errors, building nothing, if `values` is empty or a component is
    /// non-finite or negative.
    pub fn linear_rgb(values: Vec<[f64; 3]>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::LinearRgb { values },
        }
        .checked()
    }

    /// Creates a `linear-rgba` pool holding `values`, retaining ids in order.
    /// Errors, building nothing, if `values` is empty or a component is
    /// non-finite or negative.
    pub fn linear_rgba(values: Vec<[f64; 4]>) -> Result<Self> {
        let (value_ids, values) = columns(values);
        Self {
            value_ids,
            kind: VoxValuePoolKind::LinearRgba { values },
        }
        .checked()
    }

    /// Maps a freshly built pool's first flaw onto the creation errors.
    fn checked(self) -> Result<Self> {
        match self.first_flaw() {
            None => Ok(self),
            Some(VoxPoolFlaw::Empty) => Err(Error::EmptyPoolValues),
            Some(VoxPoolFlaw::Bound) => Err(Error::MalformedPoolBound),
            Some(VoxPoolFlaw::Value(value)) => Err(Error::MalformedPoolValue { value }),
        }
    }

    /// The pool's first well-formedness flaw, or `None` if it is well formed.
    /// The constructors gate on this, so a flaw found later is a voxcore bug;
    /// [`VoxMain::validate`](crate::VoxMain::validate) audits for one anyway.
    pub(crate) fn first_flaw(&self) -> Option<VoxPoolFlaw> {
        if self.value_ids.is_empty() {
            return Some(VoxPoolFlaw::Empty);
        }
        match &self.kind {
            VoxValuePoolKind::Float { min, max, .. } => {
                if !bounds_well_formed(min, max, false) {
                    return Some(VoxPoolFlaw::Bound);
                }
                for (value_id, value) in self.iter_values() {
                    let VoxPoolValueRef::Float(number) = value else {
                        unreachable!("a float pool yields float values");
                    };
                    if !number.is_finite() || !value_in_bounds(min, max, number) {
                        return Some(VoxPoolFlaw::Value(value_id));
                    }
                }
            }
            VoxValuePoolKind::Int { min, max, .. } => {
                if !bounds_well_formed(min, max, true) {
                    return Some(VoxPoolFlaw::Bound);
                }
                for (value_id, value) in self.iter_values() {
                    let VoxPoolValueRef::Int(number) = value else {
                        unreachable!("an int pool yields int values");
                    };
                    if !int_value_in_bounds(min, max, number) {
                        return Some(VoxPoolFlaw::Value(value_id));
                    }
                }
            }
            VoxValuePoolKind::Srgb { .. } | VoxValuePoolKind::Srgba { .. } => {
                return self.first_color_flaw(false);
            }
            VoxValuePoolKind::LinearRgb { .. } | VoxValuePoolKind::LinearRgba { .. } => {
                return self.first_color_flaw(true);
            }
            VoxValuePoolKind::Json { .. }
            | VoxValuePoolKind::Bool { .. }
            | VoxValuePoolKind::String { .. } => {}
        }
        None
    }

    /// The first color value with a component outside its space's range:
    /// sRGB in `[0, 1]`, linear finite and `>= 0`. The sRGB range test
    /// rejects any non-finite component on its own; the linear side is only
    /// lower-bounded, so it guards finiteness explicitly to reject
    /// `+Infinity`, which would otherwise pass `>= 0`.
    fn first_color_flaw(&self, linear: bool) -> Option<VoxPoolFlaw> {
        for (value_id, value) in self.iter_values() {
            let components: &[f64] = match value {
                VoxPoolValueRef::Srgb(color) => color,
                VoxPoolValueRef::Srgba(color) => color,
                VoxPoolValueRef::LinearRgb(color) => color,
                VoxPoolValueRef::LinearRgba(color) => color,
                _ => unreachable!("a color pool yields color values"),
            };
            for &component in components {
                let in_range = if linear {
                    component.is_finite() && component >= 0.0
                } else {
                    (0.0..=1.0).contains(&component)
                };
                if !in_range {
                    return Some(VoxPoolFlaw::Value(value_id));
                }
            }
        }
        None
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

/// Whether `min`/`max` are well formed: each numeric bound is finite (and
/// integer-valued when `integer`), and `min <= max` when both are numbers.
fn bounds_well_formed(min: &VoxBound, max: &VoxBound, integer: bool) -> bool {
    let number_ok = |bound: &VoxBound| match bound {
        VoxBound::None => true,
        VoxBound::Number(number) => number.is_finite() && !(integer && number.fract() != 0.0),
    };
    let ordered = match (min, max) {
        (VoxBound::Number(low), VoxBound::Number(high)) => low <= high,
        _ => true,
    };
    number_ok(min) && number_ok(max) && ordered
}

/// Whether `value` lies within `min`/`max`, each side unbounded when `None`.
fn value_in_bounds(min: &VoxBound, max: &VoxBound, value: f64) -> bool {
    let low_ok = match min {
        VoxBound::Number(low) => value >= *low,
        VoxBound::None => true,
    };
    let high_ok = match max {
        VoxBound::Number(high) => value <= *high,
        VoxBound::None => true,
    };
    low_ok && high_ok
}

/// Whether integer `value` lies within `min`/`max`, each side unbounded when
/// `None`. The integer sibling of [`value_in_bounds`]: it compares in the
/// integer domain, since casting an `i64` past 2^53 to `f64` rounds and could
/// carry it across a bound. Each numeric bound is finite and integer-valued,
/// which [`bounds_well_formed`] establishes first.
fn int_value_in_bounds(min: &VoxBound, max: &VoxBound, value: i64) -> bool {
    let low_ok = match min {
        VoxBound::Number(low) => int_at_least(value, *low),
        VoxBound::None => true,
    };
    let high_ok = match max {
        VoxBound::Number(high) => int_at_most(value, *high),
        VoxBound::None => true,
    };
    low_ok && high_ok
}

/// Exact `value >= bound` for a finite integer-valued `bound`. `i64::MAX as
/// f64` rounds up to 2^63 and `i64::MIN as f64` is exactly -2^63, so the two
/// range tests filter every bound outside the `i64` range; the remainder
/// converts to `i64` exactly.
fn int_at_least(value: i64, bound: f64) -> bool {
    if bound >= i64::MAX as f64 {
        false
    } else if bound < i64::MIN as f64 {
        true
    } else {
        value >= bound as i64
    }
}

/// Exact `value <= bound` for a finite integer-valued `bound`. The mirror of
/// [`int_at_least`].
fn int_at_most(value: i64, bound: f64) -> bool {
    if bound >= i64::MAX as f64 {
        true
    } else if bound < i64::MIN as f64 {
        false
    } else {
        value <= bound as i64
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
    use crate::{BVoxPoolValue, Error, VoxBound, VoxPoolValueRef, VoxValuePool};
    use branded_id::U32Id;

    fn value(index: u32) -> U32Id<BVoxPoolValue> {
        U32Id::from_u32(index)
    }

    #[test]
    fn bounded_float_pool_reads_back_in_order() {
        let pool = VoxValuePool::float(VoxBound::Number(0.0), VoxBound::None, vec![0.0, 0.5, 1.0])
            .unwrap();

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
        let pool = VoxValuePool::srgba(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap();

        assert_eq!(pool.values_len(), 1);
        assert_eq!(
            pool.value(U32Id::from_u32(0)),
            Some(VoxPoolValueRef::Srgba(&[1.0, 0.0, 0.0, 1.0]))
        );
    }

    #[test]
    fn pools_compare_by_kind_bounds_and_ordered_values() {
        let a = VoxValuePool::int(VoxBound::None, VoxBound::None, vec![1, 2]).unwrap();
        let mut b = VoxValuePool::int(VoxBound::None, VoxBound::None, vec![2, 1]).unwrap();
        assert_ne!(a, b);

        // Moving b's values into a's order makes the pools equal, even though
        // their id labels now differ per position.
        b.move_value(U32Id::from_u32(1), 0).unwrap();
        assert_eq!(a, b);

        // Same values, different bounds.
        let c = VoxValuePool::int(VoxBound::Number(0.0), VoxBound::None, vec![1, 2]).unwrap();
        assert_ne!(a, c);

        // Same shape, different kind.
        let d = VoxValuePool::srgb(vec![[0.0, 0.0, 0.0]]).unwrap();
        let e = VoxValuePool::linear_rgb(vec![[0.0, 0.0, 0.0]]).unwrap();
        assert_ne!(d, e);
    }

    #[test]
    fn move_value_reorders_the_listing_and_validates() {
        let mut pool =
            VoxValuePool::string(vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]).unwrap();
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
        let mut pool =
            VoxValuePool::string(vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]).unwrap();
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

    #[test]
    fn constructors_reject_an_empty_value_list() {
        assert_eq!(
            VoxValuePool::boolean(vec![]).unwrap_err(),
            Error::EmptyPoolValues
        );
        assert_eq!(
            VoxValuePool::srgba(vec![]).unwrap_err(),
            Error::EmptyPoolValues
        );
    }

    #[test]
    fn float_rejects_unordered_bounds() {
        assert_eq!(
            VoxValuePool::float(VoxBound::Number(1.0), VoxBound::Number(0.0), vec![0.5])
                .unwrap_err(),
            Error::MalformedPoolBound
        );
    }

    #[test]
    fn float_rejects_a_non_finite_bound() {
        assert_eq!(
            VoxValuePool::float(VoxBound::Number(f64::NAN), VoxBound::None, vec![0.5]).unwrap_err(),
            Error::MalformedPoolBound
        );
    }

    #[test]
    fn int_rejects_a_non_integer_bound() {
        assert_eq!(
            VoxValuePool::int(VoxBound::Number(0.5), VoxBound::None, vec![1]).unwrap_err(),
            Error::MalformedPoolBound
        );
    }

    #[test]
    fn float_rejects_a_value_out_of_bounds() {
        assert_eq!(
            VoxValuePool::float(VoxBound::Number(0.0), VoxBound::Number(1.0), vec![0.0, 2.0])
                .unwrap_err(),
            Error::MalformedPoolValue { value: value(1) }
        );
    }

    #[test]
    fn float_rejects_a_non_finite_value() {
        assert_eq!(
            VoxValuePool::float(VoxBound::None, VoxBound::None, vec![f64::NAN]).unwrap_err(),
            Error::MalformedPoolValue { value: value(0) }
        );
    }

    #[test]
    fn int_compares_values_against_bounds_exactly() {
        // 2^53 + 1 rounds down to 2^53 as f64, so a float comparison would
        // wrongly accept it against a max of 2^53; the integer-domain
        // comparison rejects it.
        const MAX: i64 = 1 << 53;
        assert_eq!(
            VoxValuePool::int(
                VoxBound::None,
                VoxBound::Number(MAX as f64),
                vec![MAX, MAX + 1],
            )
            .unwrap_err(),
            Error::MalformedPoolValue { value: value(1) }
        );
    }

    #[test]
    fn srgb_rejects_a_component_out_of_range() {
        assert_eq!(
            VoxValuePool::srgb(vec![[0.0, 0.0, 0.0], [0.0, 1.5, 0.0]]).unwrap_err(),
            Error::MalformedPoolValue { value: value(1) }
        );
    }

    #[test]
    fn linear_rgb_accepts_hdr_components() {
        // Linear colors allow components above 1 for HDR.
        assert!(VoxValuePool::linear_rgb(vec![[0.0, 4.0, 12.0]]).is_ok());
    }

    #[test]
    fn linear_rgba_rejects_a_negative_component() {
        assert_eq!(
            VoxValuePool::linear_rgba(vec![[0.0, -0.1, 0.0, 1.0]]).unwrap_err(),
            Error::MalformedPoolValue { value: value(0) }
        );
    }

    #[test]
    fn linear_rgb_rejects_a_non_finite_component() {
        // A linear pool is only lower-bounded, so `+Infinity` would pass a bare
        // `>= 0` test; the finiteness guard must reject it.
        assert_eq!(
            VoxValuePool::linear_rgb(vec![[0.0, f64::INFINITY, 0.0]]).unwrap_err(),
            Error::MalformedPoolValue { value: value(0) }
        );
    }
}
