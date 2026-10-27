use crate::{
    ProfileSet, ResolvePrefsPaths, Result,
    commands::{ObjectConfig, RenderProfile, built_in_render_profiles},
    load_profile_set,
};
use ty_preferences::Dependencies as PreferencesDependencies;

/// The profiles `object render` can apply. The built-ins sit under each
/// `.vxlconfig` layer's `object.render.profiles`, with the user's layer
/// first and the working directory's last.
pub fn load_render_profile_set(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<ProfileSet<RenderProfile>> {
    load_profile_set(
        dependencies,
        "object",
        built_in_render_profiles(),
        |config: ObjectConfig| config.render.profiles,
    )
}
