use crate::{ProfileSet, ResolvePrefsPaths, Result};
use serde::de::DeserializeOwned;
use std::{collections::BTreeMap, io::Error as IOError};
use ty_preferences::{Dependencies as PreferencesDependencies, JsoncCodec, load_application_prefs};

/// The `built_ins` under each `.vxlconfig` layer's profiles, which `profiles`
/// pulls from the layer's `section`. The user's layer comes first and the
/// working directory's last.
pub fn load_profile_set<C: DeserializeOwned, P>(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
    section: &str,
    built_ins: BTreeMap<String, P>,
    profiles: impl Fn(C) -> BTreeMap<String, P>,
) -> Result<ProfileSet<P>> {
    let paths = dependencies.resolve_prefs_paths()?;

    let layers =
        load_application_prefs::<C>(dependencies, &JsoncCodec, &paths, ".vxlconfig", section)
            .map_err(|error| {
                IOError::new(
                    error.kind(),
                    format!("a `.vxlconfig` layer failed to load: {error}"),
                )
            })?;

    Ok(ProfileSet::layered(
        built_ins,
        layers
            .into_iter()
            .map(|layer| (layer.dir.join(".vxlconfig"), profiles(layer.prefs))),
    ))
}
