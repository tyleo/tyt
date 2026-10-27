use crate::{Error, ProfileOrigin, Result};
use std::{collections::BTreeMap, path::PathBuf};

/// The profiles a command can apply, one namespace merged from the layers of
/// the cascade. Each name reads from the last layer supplying it, wholesale,
/// and remembers that layer as its origin.
#[derive(Clone, Debug)]
pub struct ProfileSet<P> {
    /// The layers in cascade order, the built-ins first.
    cascade: Vec<ProfileOrigin>,

    profiles: BTreeMap<String, (ProfileOrigin, P)>,
}

impl<P> ProfileSet<P> {
    /// The `built_ins` under `layers`, each a `.vxlconfig` path with its
    /// profiles, each name reading from the last layer supplying it.
    pub(crate) fn layered(
        built_ins: BTreeMap<String, P>,
        layers: impl IntoIterator<Item = (PathBuf, BTreeMap<String, P>)>,
    ) -> Self {
        let mut cascade = vec![ProfileOrigin::BuiltIn];

        let mut profiles: BTreeMap<_, _> = built_ins
            .into_iter()
            .map(|(name, profile)| (name, (ProfileOrigin::BuiltIn, profile)))
            .collect();

        for (path, layer) in layers {
            let origin = ProfileOrigin::File(path);

            profiles.extend(
                layer
                    .into_iter()
                    .map(|(name, profile)| (name, (origin.clone(), profile))),
            );

            cascade.push(origin);
        }

        ProfileSet { cascade, profiles }
    }

    /// A set holding `profiles` alone, as built-ins.
    #[cfg(test)]
    pub(crate) fn from_profiles(profiles: BTreeMap<String, P>) -> Self {
        Self::layered(profiles, [])
    }

    /// Each origin supplying a name, in cascade order, with the profiles it
    /// supplies in name order. An origin every later layer overrode is
    /// absent.
    pub(crate) fn by_origin(&self) -> Vec<(&ProfileOrigin, Vec<(&str, &P)>)> {
        self.cascade
            .iter()
            .map(|origin| {
                let profiles = self
                    .profiles
                    .iter()
                    .filter(|(_, (supplier, _))| supplier == origin)
                    .map(|(name, (_, profile))| (name.as_str(), profile))
                    .collect::<Vec<_>>();

                (origin, profiles)
            })
            .filter(|(_, profiles)| !profiles.is_empty())
            .collect()
    }

    /// The profile `name`, which `origin` asks for.
    pub(crate) fn get(&self, origin: &str, name: &str) -> Result<&P> {
        Ok(self.get_with_origin(origin, name)?.1)
    }

    /// The profile `name`, which `origin` asks for, with the origin supplying
    /// it.
    pub(crate) fn get_with_origin(&self, origin: &str, name: &str) -> Result<(&ProfileOrigin, &P)> {
        self.profiles
            .get(name)
            .map(|(supplier, profile)| (supplier, profile))
            .ok_or_else(|| {
                let names: Vec<_> = self
                    .profiles
                    .keys()
                    .map(|name| format!("`{name}`"))
                    .collect();

                let defined = if names.is_empty() {
                    "No profile is defined".to_owned()
                } else {
                    format!("The profiles are {}", names.join(", "))
                };

                Error::usage(format!(
                    "{origin} asks for the profile `{name}`, which is not defined. {defined}"
                ))
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::ProfileSet;
    use std::{collections::BTreeMap, path::PathBuf};

    /// The profiles `names`, each holding `text`.
    fn profiles(names: &[&str], text: &'static str) -> BTreeMap<String, &'static str> {
        names
            .iter()
            .map(|name| ((*name).to_owned(), text))
            .collect()
    }

    /// A layer at `path` holding the profile `name` as `text`.
    fn layer(
        path: &str,
        name: &str,
        text: &'static str,
    ) -> (PathBuf, BTreeMap<String, &'static str>) {
        (PathBuf::from(path), profiles(&[name], text))
    }

    #[test]
    fn a_later_layer_replaces_a_name_wholesale() {
        let profiles = ProfileSet::layered(
            profiles(&["pbr"], "built in"),
            [
                layer("/home/.vxlconfig", "orm", "home"),
                layer("/repo/.vxlconfig", "a", "repo"),
                layer("/repo/sub/.vxlconfig", "orm", "sub"),
            ],
        );

        assert_eq!(*profiles.get("the test", "orm").unwrap(), "sub");
        assert_eq!(*profiles.get("the test", "a").unwrap(), "repo");
        assert_eq!(*profiles.get("the test", "pbr").unwrap(), "built in");
    }

    #[test]
    fn by_origin_runs_the_cascade_and_drops_an_overridden_layer() {
        let profiles = ProfileSet::layered(
            profiles(&["albedo", "pbr"], "built in"),
            [
                layer("/home/.vxlconfig", "orm", "home"),
                layer("/repo/.vxlconfig", "a", "repo"),
                layer("/repo/sub/.vxlconfig", "orm", "sub"),
            ],
        );

        let groups: Vec<_> = profiles
            .by_origin()
            .into_iter()
            .map(|(origin, profiles)| (origin.to_string(), profiles))
            .collect();

        assert_eq!(
            groups,
            [
                (
                    "built in".to_owned(),
                    vec![("albedo", &"built in"), ("pbr", &"built in")]
                ),
                ("/repo/.vxlconfig".to_owned(), vec![("a", &"repo")]),
                ("/repo/sub/.vxlconfig".to_owned(), vec![("orm", &"sub")]),
            ]
        );
    }

    #[test]
    fn an_undefined_profile_errors_with_the_defined_ones() {
        let profiles = ProfileSet::from_profiles(profiles(&["albedo", "orm"], "built in"));

        assert!(profiles.get("--profile", "orm").is_ok());

        let error = profiles.get("--profile", "metal").unwrap_err().to_string();
        assert!(error.contains("`metal`"), "{error}");
        assert!(error.contains("`albedo`, `orm`"), "{error}");
    }

    #[test]
    fn an_empty_set_says_no_profile_is_defined() {
        let profiles = ProfileSet::<()>::from_profiles(BTreeMap::new());

        let error = profiles.get("--profile", "pbr").unwrap_err().to_string();
        assert!(error.contains("No profile is defined"), "{error}");
    }
}
