use crate::operations::profile::{ProfileListEntry, ProfileListGroup, ProfileListLayout};
use treegrid::{
    TreeGrid, TreeGridBoxHierarchyOptions, TreeGridHeaderOptions, TreeGridJsonValue,
    TreeGridJsonValueCells, TreeGridLabel, TreeGridLabelMode, TreeGridMdListsOptions,
    TreeGridNestedTableOptions, TreeGridRecordsTableOptions, TreeGridRenderBoxHierarchy,
    TreeGridRenderBoxTables, TreeGridRenderJson, TreeGridRenderMdLists, TreeGridRenderMdTables,
    TreeGridRenderTextRows, TreeGridTableShape, TreeGridTableShapeKind, TreeGridTextRowsOptions,
};

/// Renders `groups` in `layout`, one node per group in the given order over
/// its profiles, each profile beside its description when `descriptions` is
/// set. The markdown lists layout alone titles them `profiles`, putting one
/// top-level heading over a section per group.
pub fn profile_list(
    groups: &[ProfileListGroup],
    layout: ProfileListLayout,
    descriptions: bool,
) -> String {
    if !descriptions {
        return profile_names_list(groups, layout);
    }

    match layout {
        ProfileListLayout::BoxHierarchy => build_described_grid(groups, false)
            .render_box_hierarchy(&TreeGridBoxHierarchyOptions::default().with_bare_roots(true)),

        ProfileListLayout::BoxTables => {
            build_records_grid(groups).render_box_tables(TreeGridTableShapeKind::Records)
        }

        ProfileListLayout::JsonCompact => build_described_grid(groups, false).render_json_compact(),

        ProfileListLayout::JsonPretty => build_described_grid(groups, false).render_json_pretty(),

        ProfileListLayout::MdLists => build_grid(groups, Some("profiles"), true)
            .render_md_lists(&TreeGridMdListsOptions::default()),

        ProfileListLayout::MdTables => build_records_grid(groups).render_md_tables(
            &TreeGridTableShape::Records(TreeGridRecordsTableOptions::default()),
        ),

        ProfileListLayout::TextRows => build_described_grid(groups, true).render_text_rows(
            &TreeGridTextRowsOptions::default()
                .with_label(TreeGridLabelMode::Header(TreeGridHeaderOptions::default())),
        ),
    }
}

/// Renders the names of `groups` alone in `layout`.
fn profile_names_list(groups: &[ProfileListGroup], layout: ProfileListLayout) -> String {
    match layout {
        ProfileListLayout::BoxHierarchy => build_grid(groups, None, false).render_box_hierarchy(
            &TreeGridBoxHierarchyOptions::default()
                .with_bare_roots(true)
                .with_value_children(true),
        ),

        ProfileListLayout::BoxTables => {
            build_grid(groups, None, false).render_box_tables(TreeGridTableShapeKind::Nested)
        }

        ProfileListLayout::JsonCompact => build_grid(groups, None, false).render_json_compact(),

        ProfileListLayout::JsonPretty => build_grid(groups, None, false).render_json_pretty(),

        ProfileListLayout::MdLists => build_grid(groups, Some("profiles"), false)
            .render_md_lists(&TreeGridMdListsOptions::default()),

        ProfileListLayout::MdTables => build_grid(groups, None, false).render_md_tables(
            &TreeGridTableShape::Nested(TreeGridNestedTableOptions::default()),
        ),

        ProfileListLayout::TextRows => {
            build_grid(groups, None, false).render_text_rows(&TreeGridTextRowsOptions::default())
        }
    }
}

/// A bare node per group over its profiles as values, each value the
/// profile's name followed by its description when `descriptions` is set. The
/// nodes are roots, or children of a bare `title` root when one is given.
fn build_grid(
    groups: &[ProfileListGroup],
    title: Option<&str>,
    descriptions: bool,
) -> TreeGrid<TreeGridJsonValueCells> {
    let mut grid = TreeGrid::with_cells(TreeGridJsonValueCells);

    let title_id = title.map(|title| grid.retain_root(TreeGridLabel::bare(title)));

    for group in groups {
        let label = TreeGridLabel::bare(group.origin.clone());

        let group_id = match title_id {
            Some(title_id) => grid.retain_child(title_id, label),
            None => grid.retain_root(label),
        };

        for profile in &group.profiles {
            let text = match &profile.description {
                Some(description) if descriptions => format!("{}: {description}", profile.name),
                _ => profile.name.clone(),
            };

            grid.push_value(group_id, TreeGridJsonValue::new(text));
        }
    }

    grid
}

/// A bare root per group over a child per profile, the child holding the
/// profile's description as its value. An undescribed profile holds no value,
/// or an empty one when `blank` is set, for the layouts that drop a node
/// without data.
fn build_described_grid(
    groups: &[ProfileListGroup],
    blank: bool,
) -> TreeGrid<TreeGridJsonValueCells> {
    let mut grid = TreeGrid::with_cells(TreeGridJsonValueCells);

    for group in groups {
        let group_id = grid.retain_root(TreeGridLabel::bare(group.origin.clone()));

        for profile in &group.profiles {
            let profile_id = grid.retain_child(group_id, TreeGridLabel::bare(profile.name.clone()));

            if let Some(description) = &profile.description {
                grid.push_value(profile_id, TreeGridJsonValue::new(description.clone()));
            } else if blank {
                grid.push_value(profile_id, TreeGridJsonValue::new(""));
            }
        }
    }

    grid
}

/// The records the table layouts render: a bare root per group over a row
/// per profile, each row's `description` column empty for an undescribed
/// profile.
fn build_records_grid(groups: &[ProfileListGroup]) -> TreeGrid<TreeGridJsonValueCells> {
    let mut grid = TreeGrid::with_cells(TreeGridJsonValueCells);

    for group in groups {
        let group_id = grid.retain_root(TreeGridLabel::bare(group.origin.clone()));

        for ProfileListEntry { name, description } in &group.profiles {
            let row_id = grid.retain_child(group_id, TreeGridLabel::bare(name.clone()));
            let description_id = grid.retain_child(row_id, TreeGridLabel::bare("description"));

            grid.push_value(
                description_id,
                TreeGridJsonValue::new(description.as_deref().unwrap_or("")),
            );
        }
    }

    grid
}

#[cfg(test)]
mod tests {
    use crate::operations::profile::{
        ProfileListEntry, ProfileListGroup, ProfileListLayout, profile_list,
    };
    use serde_json::Value;

    /// Four built-ins, then a user layer and a repository layer. `albedo` and
    /// `matte` carry descriptions.
    fn groups() -> Vec<ProfileListGroup> {
        let group = |origin: &str, profiles: &[&str]| ProfileListGroup {
            origin: origin.to_owned(),
            profiles: profiles
                .iter()
                .map(|name| ProfileListEntry {
                    name: (*name).to_owned(),
                    description: match *name {
                        "albedo" => Some("A base color texture".to_owned()),
                        "matte" => Some("No shine".to_owned()),
                        _ => None,
                    },
                })
                .collect(),
        };

        vec![
            group("built in", &["albedo", "defaults", "emissive", "pbr"]),
            group("/home/.vxlconfig", &["matte"]),
            group("/repo/.vxlconfig", &["orm"]),
        ]
    }

    #[test]
    fn hierarchy_branches_each_group_in_order() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::BoxHierarchy, false),
            "built in\n\
             ├ albedo\n\
             ├ defaults\n\
             ├ emissive\n\
             └ pbr\n\
             \n\
             /home/.vxlconfig\n\
             └ matte\n\
             \n\
             /repo/.vxlconfig\n\
             └ orm\n"
        );
    }

    #[test]
    fn box_tables_give_each_group_a_boxed_column() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::BoxTables, false),
            "┌───┬──────────┬──────────────────┬──────────────────┐\n\
             │ # │ built in │ /home/.vxlconfig │ /repo/.vxlconfig │\n\
             ├───┼──────────┼──────────────────┼──────────────────┤\n\
             │ 0 │ albedo   │ matte            │ orm              │\n\
             ├───┼──────────┼──────────────────┼──────────────────┤\n\
             │ 1 │ defaults │                  │                  │\n\
             ├───┼──────────┼──────────────────┼──────────────────┤\n\
             │ 2 │ emissive │                  │                  │\n\
             ├───┼──────────┼──────────────────┼──────────────────┤\n\
             │ 3 │ pbr      │                  │                  │\n\
             └───┴──────────┴──────────────────┴──────────────────┘\n"
        );
    }

    #[test]
    fn lists_head_each_group_over_its_numbered_profiles() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::MdLists, false),
            "# profiles\n\
             \n\
             ## built in\n\
             \n\
             1. albedo\n\
             2. defaults\n\
             3. emissive\n\
             4. pbr\n\
             \n\
             ## /home/.vxlconfig\n\
             \n\
             1. matte\n\
             \n\
             ## /repo/.vxlconfig\n\
             \n\
             1. orm\n"
        );
    }

    #[test]
    fn rows_put_each_group_beside_its_profiles() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::TextRows, false),
            "built in         albedo defaults emissive pbr\n\
             \n\
             /home/.vxlconfig matte\n\
             \n\
             /repo/.vxlconfig orm\n"
        );
    }

    #[test]
    fn tables_give_each_group_a_column() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::MdTables, false),
            "| #   | built in | /home/.vxlconfig | /repo/.vxlconfig |\n\
             | --- | -------- | ---------------- | ---------------- |\n\
             | 0   | albedo   | matte            | orm              |\n\
             | 1   | defaults |                  |                  |\n\
             | 2   | emissive |                  |                  |\n\
             | 3   | pbr      |                  |                  |\n"
        );
    }

    #[test]
    fn json_records_each_group_with_its_profiles() {
        let compact = profile_list(&groups(), ProfileListLayout::JsonCompact, false);
        assert_eq!(
            compact,
            "[{\"label\":\"built in\",\"values\":[\"albedo\",\"defaults\",\"emissive\",\"pbr\"]},\
             {\"label\":\"/home/.vxlconfig\",\"values\":[\"matte\"]},\
             {\"label\":\"/repo/.vxlconfig\",\"values\":[\"orm\"]}]\n"
        );

        let pretty = profile_list(&groups(), ProfileListLayout::JsonPretty, false);
        assert!(pretty.lines().count() > 3, "{pretty}");
        assert_eq!(
            serde_json::from_str::<Value>(&pretty).unwrap(),
            serde_json::from_str::<Value>(&compact).unwrap()
        );
    }

    #[test]
    fn no_groups_render_nothing() {
        assert_eq!(
            profile_list(&[], ProfileListLayout::BoxHierarchy, false),
            ""
        );
        assert_eq!(profile_list(&[], ProfileListLayout::MdLists, false), "");
        assert_eq!(
            profile_list(&[], ProfileListLayout::JsonCompact, false),
            "[]\n"
        );
    }

    #[test]
    fn described_hierarchy_follows_each_name_with_its_description() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::BoxHierarchy, true),
            "built in\n\
             ├ albedo: A base color texture\n\
             ├ defaults\n\
             ├ emissive\n\
             └ pbr\n\
             \n\
             /home/.vxlconfig\n\
             └ matte: No shine\n\
             \n\
             /repo/.vxlconfig\n\
             └ orm\n"
        );
    }

    #[test]
    fn described_box_tables_give_each_group_a_record_table() {
        let tables = profile_list(&groups(), ProfileListLayout::BoxTables, true);

        assert!(
            tables.starts_with(
                "built in\n\
                 \n\
                 ┌──────────┬──────────────────────┐\n\
                 │ label    │ description          │\n\
                 ├──────────┼──────────────────────┤\n\
                 │ albedo   │ A base color texture │\n"
            ),
            "{tables}"
        );
        assert!(tables.contains("│ orm   │             │\n"), "{tables}");
    }

    #[test]
    fn described_lists_follow_each_name_with_its_description() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::MdLists, true),
            "# profiles\n\
             \n\
             ## built in\n\
             \n\
             1. albedo: A base color texture\n\
             2. defaults\n\
             3. emissive\n\
             4. pbr\n\
             \n\
             ## /home/.vxlconfig\n\
             \n\
             1. matte: No shine\n\
             \n\
             ## /repo/.vxlconfig\n\
             \n\
             1. orm\n"
        );
    }

    #[test]
    fn described_rows_head_each_group_over_a_row_per_profile() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::TextRows, true),
            "# built in\n\
             \n\
             albedo   A base color texture\n\
             \n\
             defaults\n\
             \n\
             emissive\n\
             \n\
             pbr\n\
             \n\
             # /home/.vxlconfig\n\
             \n\
             matte No shine\n\
             \n\
             # /repo/.vxlconfig\n\
             \n\
             orm\n"
        );
    }

    #[test]
    fn described_tables_give_each_group_a_record_table() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::MdTables, true),
            "# built in\n\
             \n\
             | label    | description          |\n\
             | -------- | -------------------- |\n\
             | albedo   | A base color texture |\n\
             | defaults |                      |\n\
             | emissive |                      |\n\
             | pbr      |                      |\n\
             \n\
             # /home/.vxlconfig\n\
             \n\
             | label | description |\n\
             | ----- | ----------- |\n\
             | matte | No shine    |\n\
             \n\
             # /repo/.vxlconfig\n\
             \n\
             | label | description |\n\
             | ----- | ----------- |\n\
             | orm   |             |\n"
        );
    }

    #[test]
    fn described_json_holds_each_description_as_its_profiles_value() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::JsonCompact, true),
            "[{\"label\":\"built in\",\"children\":[\
             {\"label\":\"albedo\",\"values\":[\"A base color texture\"]},\
             {\"label\":\"defaults\"},{\"label\":\"emissive\"},{\"label\":\"pbr\"}]},\
             {\"label\":\"/home/.vxlconfig\",\"children\":[\
             {\"label\":\"matte\",\"values\":[\"No shine\"]}]},\
             {\"label\":\"/repo/.vxlconfig\",\"children\":[{\"label\":\"orm\"}]}]\n"
        );
    }

    #[test]
    fn no_described_groups_render_nothing() {
        assert_eq!(profile_list(&[], ProfileListLayout::BoxHierarchy, true), "");
        assert_eq!(profile_list(&[], ProfileListLayout::MdTables, true), "");
        assert_eq!(
            profile_list(&[], ProfileListLayout::JsonCompact, true),
            "[]\n"
        );
    }
}
