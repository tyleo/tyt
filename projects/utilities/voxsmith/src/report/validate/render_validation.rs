use crate::{ValidateLayout, VoxjCheck, VoxjCheckStatus, failed_check_count};
use treegrid::{
    TreeGrid, TreeGridJsonValue, TreeGridJsonValueCells, TreeGridLabel, TreeGridRenderJson,
};

/// Renders the report over a document's spec `checks`, as
/// [`check_voxj_bytes`](crate::check_voxj_bytes) runs them, in `layout`,
/// under the document's `name`.
pub fn render_validation(checks: &[VoxjCheck], name: &str, layout: ValidateLayout) -> String {
    match layout {
        ValidateLayout::Tables => render_markdown(checks, name),
        ValidateLayout::JsonPretty => build_json_grid(checks, name).render_json_pretty(),
        ValidateLayout::JsonCompact => build_json_grid(checks, name).render_json_compact(),
    }
}

/// A file-name heading, one line per check with its result, failing checks
/// listing their messages, and a closing pass/fail summary.
fn render_markdown(checks: &[VoxjCheck], name: &str) -> String {
    let mut output = format!("# {name}\n\n");
    for check in checks {
        match &check.status {
            VoxjCheckStatus::Passed => output.push_str(&format!("- {}: pass\n", check.name)),
            VoxjCheckStatus::Unverifiable => {
                output.push_str(&format!("- {}: unverifiable\n", check.name));
            }
            VoxjCheckStatus::Failed(messages) => {
                output.push_str(&format!("- {}: fail\n", check.name));
                for message in messages {
                    output.push_str(&format!("  - {message}\n"));
                }
            }
        }
    }

    let failed = failed_check_count(checks);
    output.push('\n');
    if failed == 0 {
        output.push_str("All checks passed.\n");
    } else {
        output.push_str(&format!("{failed} check{} failed.\n", plural(failed)));
    }
    output
}

/// The report tree the JSON layouts render as the shared envelope: `name`
/// and `valid` roots, then a `checks` root with one child per check bearing
/// its status as a string value, a failed check's messages under a
/// `failures` child.
fn build_json_grid(checks: &[VoxjCheck], name: &str) -> TreeGrid<TreeGridJsonValueCells> {
    let mut grid = TreeGrid::with_cells(TreeGridJsonValueCells);
    let name_root_id = grid.retain_root(TreeGridLabel::bare("name"));
    grid.push_value(name_root_id, TreeGridJsonValue::new(name));
    let valid_root_id = grid.retain_root(TreeGridLabel::bare("valid"));
    grid.push_value(
        valid_root_id,
        TreeGridJsonValue::bool(failed_check_count(checks) == 0),
    );

    let root_id = grid.retain_root(TreeGridLabel::bare("checks"));
    for check in checks {
        let node_id = grid.retain_child(root_id, TreeGridLabel::bare(check.name));
        match &check.status {
            VoxjCheckStatus::Passed => {
                grid.push_value(node_id, TreeGridJsonValue::new("passed"));
            }
            VoxjCheckStatus::Unverifiable => {
                grid.push_value(node_id, TreeGridJsonValue::new("unverifiable"));
            }
            VoxjCheckStatus::Failed(messages) => {
                grid.push_value(node_id, TreeGridJsonValue::new("failed"));
                let failures_id = grid.retain_child(node_id, TreeGridLabel::bare("failures"));
                for message in messages {
                    grid.push_value(failures_id, TreeGridJsonValue::new(message.clone()));
                }
            }
        }
    }
    grid
}

/// The plural suffix for a count.
fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use crate::{ValidateLayout, VoxjCheck, VoxjCheckStatus, render_validation};
    use serde_json::Value;

    /// One passing, one failing (with a message), and the unverifiable check.
    fn checks() -> Vec<VoxjCheck> {
        vec![
            VoxjCheck {
                name: "version",
                status: VoxjCheckStatus::Passed,
            },
            VoxjCheck {
                name: "indices",
                status: VoxjCheckStatus::Failed(vec![
                    "object 0 references palette 5, but the document has 1 palettes".to_owned(),
                ]),
            },
            VoxjCheck {
                name: "sample-order",
                status: VoxjCheckStatus::Unverifiable,
            },
        ]
    }

    #[test]
    fn tables_lists_each_check_and_a_failure_summary() {
        let output = render_validation(&checks(), "model.voxj", ValidateLayout::Tables);
        assert_eq!(
            output,
            "# model.voxj\n\n\
             - version: pass\n\
             - indices: fail\n\
             \x20\x20- object 0 references palette 5, but the document has 1 palettes\n\
             - sample-order: unverifiable\n\
             \n1 check failed.\n"
        );
    }

    #[test]
    fn tables_reports_all_passed() {
        let checks = vec![
            VoxjCheck {
                name: "version",
                status: VoxjCheckStatus::Passed,
            },
            VoxjCheck {
                name: "sample-order",
                status: VoxjCheckStatus::Unverifiable,
            },
        ];
        let output = render_validation(&checks, "ok.voxj", ValidateLayout::Tables);
        assert!(output.ends_with("\nAll checks passed.\n"));
    }

    #[test]
    fn json_compact_reports_validity_and_each_status() {
        let output = render_validation(&checks(), "model.voxj", ValidateLayout::JsonCompact);
        assert_eq!(
            output,
            "[{\"label\":\"name\",\"values\":[\"model.voxj\"]},\
             {\"label\":\"valid\",\"values\":[false]},\
             {\"label\":\"checks\",\"children\":[\
             {\"label\":\"version\",\"values\":[\"passed\"]},\
             {\"label\":\"indices\",\"values\":[\"failed\"],\"children\":[\
             {\"label\":\"failures\",\"values\":\
             [\"object 0 references palette 5, but the document has 1 palettes\"]}]},\
             {\"label\":\"sample-order\",\"values\":[\"unverifiable\"]}]}]\n"
        );
    }

    #[test]
    fn json_pretty_is_multiline_and_matches_compact() {
        let pretty = render_validation(&checks(), "model.voxj", ValidateLayout::JsonPretty);
        let compact = render_validation(&checks(), "model.voxj", ValidateLayout::JsonCompact);
        assert!(pretty.starts_with("[\n"));
        let pretty_value: Value = serde_json::from_str(&pretty).unwrap();
        let compact_value: Value = serde_json::from_str(&compact).unwrap();
        assert_eq!(pretty_value, compact_value);
    }
}
