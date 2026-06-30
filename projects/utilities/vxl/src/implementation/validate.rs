use crate::{ReportLayout, Result, implementation};
use serde_json::{Map, Value, json};
use std::{
    fs,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use voxj_codec::{VoxjCheck, VoxjCheckStatus, check_voxj_file, from_voxj_or_voxjz_file_bytes};

/// Loads the Voxel Json document at `input`, runs every spec check, writes the
/// report in `layout` to standard output, and fails when any check failed so
/// the process exits non-zero. The document is read as raw Voxel Json rather
/// than through voxcore, since the checks inspect the on-disk encoding.
pub fn validate(input: &Path, layout: ReportLayout) -> Result<()> {
    let file = from_voxj_or_voxjz_file_bytes(&fs::read(input)?)?;
    let checks = check_voxj_file(&file);
    let output = render(&checks, &file_name(input), layout)?;
    implementation::write_stdout(output.as_bytes())?;

    let failed = failed_count(&checks);
    if failed > 0 {
        // The report is already on stdout; exit non-zero with a terse summary.
        return Err(IOError::new(
            ErrorKind::InvalidData,
            format!("{failed} validation check{} failed", plural(failed)),
        )
        .into());
    }
    Ok(())
}

/// The input's file name for the report heading, or its full path when it has
/// none.
fn file_name(input: &Path) -> String {
    input
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| input.display().to_string())
}

/// Renders the report in `layout`, the testable core of [`validate`].
fn render(checks: &[VoxjCheck], name: &str, layout: ReportLayout) -> Result<String> {
    match layout {
        ReportLayout::Markdown => Ok(render_markdown(checks, name)),
        ReportLayout::PrettyJson => render_json(checks, name, true),
        ReportLayout::CompactJson => render_json(checks, name, false),
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

    let failed = failed_count(checks);
    output.push('\n');
    if failed == 0 {
        output.push_str("All checks passed.\n");
    } else {
        output.push_str(&format!("{failed} check{} failed.\n", plural(failed)));
    }
    output
}

/// The report as one JSON object: the document `name`, whether it is `valid`
/// (no check failed), and one entry per check with its `status` and, when
/// failed, its `failures`. Keys keep insertion order.
fn render_json(checks: &[VoxjCheck], name: &str, pretty: bool) -> Result<String> {
    let entries: Vec<Value> = checks
        .iter()
        .map(|check| {
            let mut entry = Map::new();
            entry.insert("name".to_owned(), json!(check.name));
            match &check.status {
                VoxjCheckStatus::Passed => {
                    entry.insert("status".to_owned(), json!("passed"));
                }
                VoxjCheckStatus::Unverifiable => {
                    entry.insert("status".to_owned(), json!("unverifiable"));
                }
                VoxjCheckStatus::Failed(messages) => {
                    entry.insert("status".to_owned(), json!("failed"));
                    entry.insert("failures".to_owned(), json!(messages));
                }
            }
            Value::Object(entry)
        })
        .collect();

    let mut document = Map::new();
    document.insert("name".to_owned(), json!(name));
    document.insert("valid".to_owned(), json!(failed_count(checks) == 0));
    document.insert("checks".to_owned(), Value::Array(entries));
    let document = Value::Object(document);

    let mut output = if pretty {
        serde_json::to_string_pretty(&document)
    } else {
        serde_json::to_string(&document)
    }
    .map_err(IOError::other)?;
    output.push('\n');
    Ok(output)
}

/// How many checks failed.
fn failed_count(checks: &[VoxjCheck]) -> usize {
    checks
        .iter()
        .filter(|check| matches!(check.status, VoxjCheckStatus::Failed(_)))
        .count()
}

/// The plural suffix for a count.
fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use crate::{ReportLayout, implementation::validate::render};
    use serde_json::Value;
    use voxj_codec::{VoxjCheck, VoxjCheckStatus};

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
    fn markdown_lists_each_check_and_a_failure_summary() {
        let output = render(&checks(), "model.voxj", ReportLayout::Markdown).unwrap();
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
    fn markdown_reports_all_passed() {
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
        let output = render(&checks, "ok.voxj", ReportLayout::Markdown).unwrap();
        assert!(output.ends_with("\nAll checks passed.\n"));
    }

    #[test]
    fn compact_json_reports_validity_and_each_status() {
        let output = render(&checks(), "model.voxj", ReportLayout::CompactJson).unwrap();
        assert_eq!(
            output,
            "{\"name\":\"model.voxj\",\"valid\":false,\"checks\":[\
             {\"name\":\"version\",\"status\":\"passed\"},\
             {\"name\":\"indices\",\"status\":\"failed\",\"failures\":\
             [\"object 0 references palette 5, but the document has 1 palettes\"]},\
             {\"name\":\"sample-order\",\"status\":\"unverifiable\"}]}\n"
        );
    }

    #[test]
    fn pretty_json_is_multiline_and_matches_compact() {
        let pretty = render(&checks(), "model.voxj", ReportLayout::PrettyJson).unwrap();
        let compact = render(&checks(), "model.voxj", ReportLayout::CompactJson).unwrap();
        assert!(pretty.starts_with("{\n"));
        let pretty_value: Value = serde_json::from_str(&pretty).unwrap();
        let compact_value: Value = serde_json::from_str(&compact).unwrap();
        assert_eq!(pretty_value, compact_value);
    }
}
