use crate::{Error, Result};

/// Pushes `item` onto `items` unless one shares its `key`, which
/// `duplicate` describes in the usage error.
pub(crate) fn push_unique<T, K: PartialEq>(
    items: &mut Vec<T>,
    item: T,
    key: impl Fn(&T) -> K,
    duplicate: impl FnOnce(&T) -> String,
) -> Result<()> {
    if let Some(existing) = items.iter().find(|existing| key(existing) == key(&item)) {
        return Err(Error::usage(duplicate(existing)));
    }

    items.push(item);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::commands::push_unique;

    #[test]
    fn a_new_key_pushes_and_a_seen_key_errors() {
        let mut items = vec![("a", 1)];

        assert!(push_unique(&mut items, ("b", 2), |item| item.0, |_| String::new()).is_ok());
        assert!(push_unique(&mut items, ("a", 3), |item| item.0, |_| String::new()).is_err());
        assert_eq!(items, [("a", 1), ("b", 2)]);
    }
}
