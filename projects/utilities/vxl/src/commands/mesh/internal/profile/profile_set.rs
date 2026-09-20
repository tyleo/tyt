use crate::{
    Error, Result,
    commands::{Profile, built_in_profiles},
};
use std::collections::BTreeMap;

/// The profiles a run can apply, one namespace merged from the layers of the
/// stack. Each name reads from the last layer supplying it, wholesale.
pub(crate) struct ProfileSet {
    profiles: BTreeMap<String, Profile>,
}

impl ProfileSet {
    /// The built-ins alone, the bottom layer of the stack.
    pub(crate) fn built_in() -> Self {
        ProfileSet {
            profiles: built_in_profiles(),
        }
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

    #[test]
    fn an_undefined_profile_errors_with_the_defined_ones() {
        let profiles = ProfileSet::built_in();

        assert!(profiles.get("--profile", "orm").is_ok());

        let error = profiles.get("--profile", "metal").unwrap_err().to_string();
        assert!(error.contains("`metal`"), "{error}");
        assert!(error.contains("`albedo`, `defaults`"), "{error}");
    }
}
