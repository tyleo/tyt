use crate::{Error, Result};

/// Wires a child command into a subcommand-enum source file: imports its type,
/// adds its `#[command(name = "...")]` variant, and adds its `match self` arm,
/// each at its sorted position.
///
/// Purely additive: existing variants, arms, imports, comments, and formatting
/// are preserved. A child already wired is left unchanged.
pub fn insert_enum_variant(contents: &str, name: &str, command: &str) -> Result<String> {
    let contents = insert_commands_use(contents, name);

    let enum_name = contents
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("pub enum ")
                .and_then(|rest| rest.split([' ', '{']).next())
        })
        .ok_or_else(|| Error::Meta("no `pub enum` found in the subcommand enum file".into()))?
        .to_string();

    let deps_param = contents
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("pub fn execute(self,")
                .and_then(|rest| rest.split(':').next())
                .map(str::trim)
        })
        .unwrap_or("dependencies")
        .to_string();

    let binding = command.replace('-', "_");
    let trailing_newline = contents.is_empty() || contents.ends_with('\n');
    let mut lines: Vec<String> = contents.lines().map(str::to_string).collect();

    let variant = [
        format!("    #[command(name = \"{command}\")]"),
        format!("    {name}({name}),"),
    ];
    insert_into_braces(
        &mut lines,
        |line| line.starts_with("pub enum "),
        variant_key,
        name,
        &variant,
    )
    .ok_or_else(|| Error::Meta("could not place the enum variant".into()))?;

    let arm = [format!(
        "            {enum_name}::{name}({binding}) => {binding}.execute({deps_param}),"
    )];
    insert_into_braces(
        &mut lines,
        |line| line.starts_with("match self"),
        arm_key,
        name,
        &arm,
    )
    .ok_or_else(|| Error::Meta("could not place the match arm".into()))?;

    let mut result = lines.join("\n");
    if trailing_newline {
        result.push('\n');
    }
    Ok(result)
}

/// The variant name in an enum-body line such as `Foo(Foo),`, or `None` for an
/// attribute, blank, or brace line.
fn variant_key(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("#[") || trimmed == "}" {
        return None;
    }
    Some(
        trimmed
            .split(['(', ',', ' ', '{'])
            .next()
            .unwrap_or(trimmed),
    )
}

/// The variant name in a match arm such as `Enum::Foo(foo) => ...`, or `None`
/// for an attribute, blank, or brace line.
fn arm_key(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("#[") || trimmed == "}" {
        return None;
    }
    let after = trimmed.split("::").nth(1)?;
    Some(after.split('(').next().unwrap_or(after))
}

/// Splices `name` into the file's `use crate::commands::{...}` import at its
/// sorted position, tolerant of single- or multi-line brace lists. Adds the
/// import when absent and leaves it unchanged when `name` is already present.
fn insert_commands_use(contents: &str, name: &str) -> String {
    let trailing_newline = contents.is_empty() || contents.ends_with('\n');
    let mut lines: Vec<String> = contents.lines().map(str::to_string).collect();

    let render = |lines: Vec<String>| {
        let mut result = lines.join("\n");
        if trailing_newline {
            result.push('\n');
        }
        result
    };

    let Some(start) = lines
        .iter()
        .position(|line| line.trim_start().starts_with("use crate::commands::"))
    else {
        let at = lines
            .iter()
            .position(|line| line.trim_start().starts_with("use "))
            .unwrap_or(0);
        lines.insert(at, format!("use crate::commands::{{{name}}};"));
        return render(lines);
    };

    // The statement runs from `start` to the first line closing it with `;`.
    let end = (start..lines.len())
        .find(|&index| lines[index].contains(';'))
        .unwrap_or(start);

    // Collect the names across however many lines the brace list spans.
    let statement = lines[start..=end].join("\n");
    let inner = statement
        .trim()
        .trim_start_matches("use crate::commands::")
        .trim_end_matches(';')
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}');
    let mut names: Vec<String> = inner
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(String::from)
        .collect();

    if names.iter().any(|existing| existing == name) {
        return render(lines);
    }

    names.push(name.to_string());
    names.sort();
    lines.splice(
        start..=end,
        [format!("use crate::commands::{{{}}};", names.join(", "))],
    );
    render(lines)
}

/// Inserts `entry` (its lines already indented) into the brace-delimited body
/// whose opening line matches `header`, keeping entries sorted by `key`. Handles
/// an empty `{}` body by expanding it. Returns `None` when no header line is
/// found, `Some(())` otherwise (including a no-op when `new_key` is present).
fn insert_into_braces(
    lines: &mut Vec<String>,
    header: impl Fn(&str) -> bool,
    key: impl Fn(&str) -> Option<&str>,
    new_key: &str,
    entry: &[String],
) -> Option<()> {
    let open = lines.iter().position(|line| header(line.trim()))?;
    let indent: String = lines[open]
        .chars()
        .take_while(|character| character.is_whitespace())
        .collect();

    // An empty `{}` body: expand it around the new entry.
    if let Some(brace) = lines[open].find("{}") {
        let head = lines[open][..brace].to_string();
        let mut replacement = vec![format!("{head}{{")];
        replacement.extend(entry.iter().cloned());
        replacement.push(format!("{indent}}}"));
        lines.splice(open..=open, replacement);
        return Some(());
    }

    // Find the matching close brace by tracking depth from the opening line.
    let mut depth = 0i32;
    let mut close = open;
    'outer: for (index, line) in lines.iter().enumerate().skip(open) {
        for character in line.chars() {
            match character {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        close = index;
                        break 'outer;
                    }
                }
                _ => {}
            }
        }
    }

    // Already wired: leave it be.
    if lines[open + 1..close]
        .iter()
        .filter_map(|line| key(line))
        .any(|existing| existing == new_key)
    {
        return Some(());
    }

    // Insert before the first later entry (above its attributes), else before
    // the closing brace.
    let mut insert_at = close;
    for index in open + 1..close {
        if let Some(found) = key(&lines[index])
            && found > new_key
        {
            let mut at = index;
            while at > open + 1 && lines[at - 1].trim_start().starts_with("#[") {
                at -= 1;
            }
            insert_at = at;
            break;
        }
    }

    for (offset, line) in entry.iter().enumerate() {
        lines.insert(insert_at + offset, line.clone());
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use crate::commands::create_command::insert_enum_variant;

    /// An empty subcommand enum as `group_enum_template` renders it.
    fn empty_enum() -> String {
        "use clap::Subcommand;\n\n/// The `to` command group.\n#[derive(Clone, Debug, Subcommand)]\n#[command(subcommand_value_name = \"command\")]\npub enum ToCommand {}\n\nimpl ToCommand {\n    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {\n        match self {}\n    }\n}\n".to_string()
    }

    #[test]
    fn wires_the_first_child_into_an_empty_enum() {
        let wired = insert_enum_variant(&empty_enum(), "ToGoxl", "goxl").unwrap();
        assert!(wired.contains("use crate::commands::{ToGoxl};"));
        assert!(wired.contains("    #[command(name = \"goxl\")]\n    ToGoxl(ToGoxl),\n"));
        assert!(
            wired.contains("            ToCommand::ToGoxl(goxl) => goxl.execute(_dependencies),\n")
        );
        // The empty braces are gone.
        assert!(!wired.contains("pub enum ToCommand {}"));
        assert!(!wired.contains("match self {}"));
    }

    #[test]
    fn inserts_a_second_child_in_sorted_position() {
        let once = insert_enum_variant(&empty_enum(), "ToVmax", "vmax").unwrap();
        let twice = insert_enum_variant(&once, "ToGoxl", "goxl").unwrap();
        // Import stays a single sorted list.
        assert!(twice.contains("use crate::commands::{ToGoxl, ToVmax};"));
        // Goxl sorts before Vmax in both the enum and the match.
        let goxl = twice.find("ToGoxl(ToGoxl)").unwrap();
        let vmax = twice.find("ToVmax(ToVmax)").unwrap();
        assert!(goxl < vmax);
        let goxl_arm = twice.find("ToCommand::ToGoxl").unwrap();
        let vmax_arm = twice.find("ToCommand::ToVmax").unwrap();
        assert!(goxl_arm < vmax_arm);
    }

    #[test]
    fn splices_into_a_multiline_import_by_comma_not_line() {
        // rustfmt may reflow the import across lines between runs; the splice must
        // still find the comma-separated names.
        let source = "use crate::commands::{\n    ToGoxl, ToVmax, ToVoxj,\n};\nuse clap::Subcommand;\n\n#[derive(Clone, Debug, Subcommand)]\npub enum ToCommand {\n    #[command(name = \"goxl\")]\n    ToGoxl(ToGoxl),\n    #[command(name = \"vmax\")]\n    ToVmax(ToVmax),\n    #[command(name = \"voxj\")]\n    ToVoxj(ToVoxj),\n}\n\nimpl ToCommand {\n    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {\n        match self {\n            ToCommand::ToGoxl(goxl) => goxl.execute(dependencies),\n            ToCommand::ToVmax(vmax) => vmax.execute(dependencies),\n            ToCommand::ToVoxj(voxj) => voxj.execute(dependencies),\n        }\n    }\n}\n".to_string();
        let wired = insert_enum_variant(&source, "ToMvox", "mvox").unwrap();
        assert!(wired.contains("use crate::commands::{ToGoxl, ToMvox, ToVmax, ToVoxj};"));
        assert!(wired.contains("    #[command(name = \"mvox\")]\n    ToMvox(ToMvox),\n"));
        assert!(wired.contains("ToCommand::ToMvox(mvox) => mvox.execute(dependencies),"));
    }

    #[test]
    fn re_wiring_an_existing_child_is_a_no_op() {
        let once = insert_enum_variant(&empty_enum(), "ToGoxl", "goxl").unwrap();
        let twice = insert_enum_variant(&once, "ToGoxl", "goxl").unwrap();
        assert_eq!(once, twice);
    }
}
