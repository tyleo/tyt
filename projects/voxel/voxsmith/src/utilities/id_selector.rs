use crate::{Error, Result};
use branded_id::U32Id;
use std::{fmt, ops::RangeInclusive};

/// Selects a document's `TBrand` entries by id: every entry (`*`), one id
/// (`#`), or an inclusive id range (`#-#`). The default selects every entry.
pub struct IdSelector<TBrand> {
    /// The selected ids, or `None` for every entry.
    range: Option<RangeInclusive<U32Id<TBrand>>>,
}

impl<TBrand> IdSelector<TBrand> {
    /// Selects every entry.
    pub fn all() -> Self {
        Self { range: None }
    }

    /// Selects the entry `id`.
    pub fn id(id: U32Id<TBrand>) -> Self {
        Self {
            range: Some(id..=id),
        }
    }

    /// Selects the entries `range` spans. Errors when its start comes after
    /// its end.
    pub fn range(range: RangeInclusive<U32Id<TBrand>>) -> Result<Self> {
        if range.start() > range.end() {
            return Err(Error::invalid(format!(
                "range start {} is greater than its end {}",
                range.start(),
                range.end()
            )));
        }

        Ok(Self { range: Some(range) })
    }

    /// Whether the selector selects every entry.
    pub fn is_all(&self) -> bool {
        self.range.is_none()
    }

    /// Whether the selector selects the entry `id`.
    pub fn contains(&self, id: U32Id<TBrand>) -> bool {
        let Some(range) = &self.range else {
            return true;
        };

        range.contains(&id)
    }

    /// The ids the selector names, or `None` when it selects every entry.
    pub fn as_range(&self) -> Option<&RangeInclusive<U32Id<TBrand>>> {
        self.range.as_ref()
    }
}

impl<TBrand> Clone for IdSelector<TBrand> {
    fn clone(&self) -> Self {
        Self {
            range: self.range.clone(),
        }
    }
}

impl<TBrand> fmt::Debug for IdSelector<TBrand> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("IdSelector").field(&self.range).finish()
    }
}

impl<TBrand> Default for IdSelector<TBrand> {
    fn default() -> Self {
        Self::all()
    }
}

impl<TBrand> Eq for IdSelector<TBrand> {}

impl<TBrand> PartialEq for IdSelector<TBrand> {
    fn eq(&self, other: &Self) -> bool {
        self.range == other.range
    }
}

#[cfg(test)]
mod tests {
    use crate::utilities::IdSelector;
    use branded_id::{RangeInclusiveExt, U32Id};

    struct BEntry;

    fn id(value: u32) -> U32Id<BEntry> {
        U32Id::from_u32(value)
    }

    #[test]
    fn all_selects_every_id_and_names_none() {
        let selector = IdSelector::<BEntry>::all();

        assert!(selector.is_all());

        assert!(selector.contains(U32Id::MIN));

        assert!(selector.contains(U32Id::MAX));

        assert!(selector.as_range().is_none());
    }

    #[test]
    fn an_id_selects_itself_alone() {
        let selector = IdSelector::id(id(5));

        assert!(!selector.is_all());

        assert!(selector.contains(id(5)));

        assert!(!selector.contains(id(4)));

        assert!(!selector.contains(id(6)));

        assert_eq!(selector.as_range(), Some(&(id(5)..=id(5))));
    }

    #[test]
    fn a_range_selects_and_names_its_inclusive_span() {
        let selector = IdSelector::range(id(2)..=id(4)).unwrap();

        assert!(!selector.contains(id(1)));

        assert!(selector.contains(id(2)));

        assert!(selector.contains(id(4)));

        assert!(!selector.contains(id(5)));

        let range = selector.as_range().unwrap().clone();

        assert_eq!(
            range.into_id_range().collect::<Vec<_>>(),
            [id(2), id(3), id(4)]
        );
    }

    #[test]
    fn a_range_reaches_the_largest_id() {
        let selector = IdSelector::range(U32Id::MAX..=U32Id::MAX).unwrap();

        assert!(selector.contains(U32Id::<BEntry>::MAX));
    }

    #[test]
    fn a_range_of_one_equals_the_id() {
        assert_eq!(
            IdSelector::range(id(3)..=id(3)).unwrap(),
            IdSelector::id(id(3))
        );
    }

    #[test]
    fn a_reversed_range_errors() {
        assert_eq!(
            IdSelector::range(id(5)..=id(2)).unwrap_err().to_string(),
            "range start 5 is greater than its end 2"
        );
    }

    #[test]
    fn the_default_selects_every_id() {
        assert_eq!(IdSelector::<BEntry>::default(), IdSelector::all());
    }
}
