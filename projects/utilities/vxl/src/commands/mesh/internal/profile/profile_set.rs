use crate::{
    Error, Result,
    commands::{Profile, built_in_profiles},
};
use std::collections::BTreeMap;

/// The profiles a run can apply, one namespace merged from the layers of the
/// stack. Each name reads from the last layer supplying it, wholesale.
#[derive(Clone, Debug)]
pub(crate) struct ProfileSet {
    profiles: BTreeMap<String, Profile>,
}

impl ProfileSet {
    /// The built-ins alone, the bottom layer of the stack.
    #[cfg(test)]
    pub(crate) fn built_in() -> Self {
        ProfileSet {
            profiles: built_in_profiles(),
        }
    }

    /// The built-ins under `layers`, each name reading from the last layer
    /// supplying it.
    pub(crate) fn layered(layers: impl IntoIterator<Item = BTreeMap<String, Profile>>) -> Self {
        let mut profiles = built_in_profiles();

        for layer in layers {
            profiles.extend(layer);
        }

        ProfileSet { profiles }
    }

    /// A set holding `profiles` alone.
    #[cfg(test)]
    pub(crate) fn from_profiles(profiles: BTreeMap<String, Profile>) -> Self {
        ProfileSet { profiles }
    }

    /// The profile `name`, which `origin` asks for.
    pub(crate) fn get(&self, origin: &str, name: &str) -> Result<&Profile> {
        self.profiles.get(name).ok_or_else(|| {
            let names: Vec<_> = self
                .profiles
                .keys()
                .map(|name| format!("`{name}`"))
                .collect();

            Error::usage(format!(
                "{origin} asks for the profile `{name}`, which is not defined. The profiles are {}",
                names.join(", ")
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ProfileSet;
    use crate::commands::Profile;
    use std::collections::BTreeMap;

    /// A layer holding the profile `name` with `values`.
    fn layer(name: &str, values: &[&str]) -> BTreeMap<String, Profile> {
        let profile = Profile {
            values: values.iter().map(|value| (*value).to_owned()).collect(),
            ..Profile::default()
        };

        BTreeMap::from([(name.to_owned(), profile)])
    }

    #[test]
    fn a_later_layer_replaces_a_name_wholesale() {
        let profiles = ProfileSet::layered([
            layer("orm", &["orm = 1"]),
            layer("a", &["a = 1"]),
            layer("orm", &["orm = 2"]),
        ]);

        let orm = profiles.get("the test", "orm").unwrap();
        assert_eq!(orm.values, ["orm = 2"]);
        assert!(orm.materials.is_empty());
        assert!(profiles.get("the test", "a").is_ok());
        assert!(profiles.get("the test", "pbr").is_ok());
    }

    #[test]
    fn an_undefined_profile_errors_with_the_defined_ones() {
        let profiles = ProfileSet::built_in();

        assert!(profiles.get("--profile", "orm").is_ok());

        let error = profiles.get("--profile", "metal").unwrap_err().to_string();
        assert!(error.contains("`metal`"), "{error}");
        assert!(error.contains("`albedo`, `defaults`"), "{error}");
    }
}
