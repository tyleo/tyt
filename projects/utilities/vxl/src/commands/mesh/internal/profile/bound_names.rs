use serde::Deserialize;

/// The names a profile's compute key binds, written as one string or a list.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(from = "BoundNamesRepr")]
pub(crate) struct BoundNames(pub(crate) Vec<String>);

#[derive(Deserialize)]
#[serde(untagged)]
enum BoundNamesRepr {
    List(Vec<String>),
    One(String),
}

impl From<BoundNamesRepr> for BoundNames {
    fn from(repr: BoundNamesRepr) -> Self {
        match repr {
            BoundNamesRepr::List(names) => BoundNames(names),
            BoundNamesRepr::One(name) => BoundNames(vec![name]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BoundNames;

    #[test]
    fn a_string_or_a_list_binds_names() {
        assert_eq!(
            serde_json::from_str::<BoundNames>("\"ao\"").unwrap(),
            BoundNames(vec!["ao".to_owned()])
        );
        assert_eq!(
            serde_json::from_str::<BoundNames>("[\"ao\", \"at\"]").unwrap(),
            BoundNames(vec!["ao".to_owned(), "at".to_owned()])
        );
        assert!(serde_json::from_str::<BoundNames>("1").is_err());
    }
}
