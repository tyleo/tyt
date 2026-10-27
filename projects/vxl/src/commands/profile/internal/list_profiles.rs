use crate::{Profile, ProfileSet};
use voxsmith::operations::profile::{
    ProfileListEntry, ProfileListGroup, ProfileListLayout, profile_list,
};

/// The profiles of `profiles` in `layout`: one group per origin in cascade
/// order, each holding the profiles it supplies in name order, beside their
/// descriptions when `descriptions` is set.
pub fn list_profiles<P: Profile>(
    profiles: &ProfileSet<P>,
    layout: ProfileListLayout,
    descriptions: bool,
) -> String {
    let groups: Vec<_> = profiles
        .by_origin()
        .into_iter()
        .map(|(origin, profiles)| ProfileListGroup {
            origin: origin.to_string(),
            profiles: profiles
                .into_iter()
                .map(|(name, profile)| ProfileListEntry {
                    name: name.to_owned(),
                    description: profile
                        .description()
                        .map(|description| description.as_str().to_owned()),
                })
                .collect(),
        })
        .collect();

    profile_list(&groups, layout, descriptions)
}

#[cfg(test)]
mod tests {
    use crate::{ProfileSet, QuantizeProfile, commands::list_profiles};
    use std::{collections::BTreeMap, path::PathBuf};
    use voxsmith::operations::profile::ProfileListLayout;

    /// The profiles `entries` names, each holding its json.
    fn layer(entries: &[(&str, &str)]) -> BTreeMap<String, QuantizeProfile> {
        entries
            .iter()
            .map(|(name, json)| ((*name).to_owned(), serde_json::from_str(json).unwrap()))
            .collect()
    }

    /// Three built-ins under a user layer and a repository layer, the
    /// repository's `orm` overriding the built-in.
    fn profiles() -> ProfileSet<QuantizeProfile> {
        ProfileSet::layered(
            layer(&[
                ("albedo", r#"{ "description": "Eight base colors" }"#),
                ("orm", "{}"),
                ("pbr", "{}"),
            ]),
            [
                (
                    PathBuf::from("/home/.vxlconfig"),
                    layer(&[("matte", r#"{ "description": "No shine" }"#)]),
                ),
                (PathBuf::from("/repo/.vxlconfig"), layer(&[("orm", "{}")])),
            ],
        )
    }

    #[test]
    fn the_origins_group_in_cascade_order() {
        assert_eq!(
            list_profiles(&profiles(), ProfileListLayout::TextRows, false),
            "built in         albedo pbr\n\
             \n\
             /home/.vxlconfig matte\n\
             \n\
             /repo/.vxlconfig orm\n"
        );
    }

    #[test]
    fn each_description_follows_its_profile() {
        assert_eq!(
            list_profiles(&profiles(), ProfileListLayout::BoxHierarchy, true),
            "built in\n\
             ├ albedo: Eight base colors\n\
             └ pbr\n\
             \n\
             /home/.vxlconfig\n\
             └ matte: No shine\n\
             \n\
             /repo/.vxlconfig\n\
             └ orm\n"
        );
    }
}
