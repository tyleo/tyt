use crate::ProfileSet;
use serde::de::DeserializeOwned;

/// A set holding the profiles `entries` defines as json.
pub fn profile_set_from_json<P: DeserializeOwned>(entries: &[(&str, &str)]) -> ProfileSet<P> {
    ProfileSet::from_profiles(
        entries
            .iter()
            .map(|(name, json)| ((*name).to_owned(), serde_json::from_str(json).unwrap()))
            .collect(),
    )
}
