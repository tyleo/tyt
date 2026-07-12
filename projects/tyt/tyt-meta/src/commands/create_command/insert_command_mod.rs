/// Inserts a `mod {module};` and a `pub use {module}::*;` line into a
/// `commands/mod.rs`, each at its sorted position, and returns the new source.
///
/// Purely additive: every existing line is preserved verbatim, including doc
/// comments, blank lines, `#[allow(...)]` attributes, and `pub(crate) use`
/// re-exports. If `module` is already declared, the source is returned
/// unchanged.
pub fn insert_command_mod(contents: &str, module: &str) -> String {
    let mut lines: Vec<String> = contents.lines().map(str::to_string).collect();
    let trailing_newline = contents.is_empty() || contents.ends_with('\n');
    let has_pub_use = lines.iter().any(|line| pub_use_name(line).is_some());

    insert_sorted_line(&mut lines, &format!("mod {module};"), module, mod_name);
    // A freshly scaffolded commands/mod.rs starts empty; give it the two-section
    // blank separator before its first re-export.
    if !has_pub_use && lines.last().is_some_and(|line| !line.trim().is_empty()) {
        lines.push(String::new());
    }
    insert_sorted_line(
        &mut lines,
        &format!("pub use {module}::*;"),
        module,
        pub_use_name,
    );

    let mut result = lines.join("\n");
    if trailing_newline {
        result.push('\n');
    }
    result
}

/// The module name in a `mod {name};` line, ignoring leading whitespace.
fn mod_name(line: &str) -> Option<&str> {
    line.trim().strip_prefix("mod ")?.strip_suffix(';')
}

/// The module name in a `pub use {name}::*;` line, ignoring leading whitespace.
fn pub_use_name(line: &str) -> Option<&str> {
    line.trim().strip_prefix("pub use ")?.strip_suffix("::*;")
}

/// Inserts `new_line` among the lines matching `key`, keeping them sorted by the
/// key. Skips insertion when `name` is already present. An insertion before an
/// attributed line lands above the attribute so it stays with its module.
fn insert_sorted_line(
    lines: &mut Vec<String>,
    new_line: &str,
    name: &str,
    key: impl Fn(&str) -> Option<&str>,
) {
    let matches: Vec<(usize, &str)> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| key(line).map(|found| (index, found)))
        .collect();

    if matches.iter().any(|(_, found)| *found == name) {
        return;
    }

    let insert_at = match matches.iter().find(|(_, found)| *found > name) {
        // Before the first later peer, above any attributes bound to it.
        Some(&(index, _)) => {
            let mut at = index;
            while at > 0 && lines[at - 1].trim_start().starts_with("#[") {
                at -= 1;
            }
            at
        }
        // After the last peer, or at the end when there are none.
        None => matches
            .last()
            .map(|&(index, _)| index + 1)
            .unwrap_or(lines.len()),
    };

    lines.insert(insert_at, new_line.to_string());
}

#[cfg(test)]
mod tests {
    use crate::commands::create_command::insert_command_mod;

    #[test]
    fn inserts_a_mod_and_pub_use_in_sorted_position() {
        let source = "mod alpha;\nmod gamma;\n\npub use alpha::*;\npub use gamma::*;\n";
        let expected = "mod alpha;\nmod beta;\nmod gamma;\n\npub use alpha::*;\npub use beta::*;\npub use gamma::*;\n";
        assert_eq!(insert_command_mod(source, "beta"), expected);
    }

    #[test]
    fn appends_when_alphabetically_last() {
        let source = "mod alpha;\n\npub use alpha::*;\n";
        let expected = "mod alpha;\nmod zed;\n\npub use alpha::*;\npub use zed::*;\n";
        assert_eq!(insert_command_mod(source, "zed"), expected);
    }

    #[test]
    fn preserves_module_doc_attributes_and_pub_crate_use() {
        // A nested group mod.rs: an inner doc line, a module_inception attribute,
        // and a pub(crate) re-export must all survive untouched.
        let source = "//! Module docs.\n\n#[allow(clippy::module_inception)]\nmod to;\nmod to_goxl;\n\npub use to::*;\npub(crate) use to_goxl::*;\n";
        let expected = "//! Module docs.\n\n#[allow(clippy::module_inception)]\nmod to;\nmod to_command;\nmod to_goxl;\n\npub use to::*;\npub use to_command::*;\npub(crate) use to_goxl::*;\n";
        assert_eq!(insert_command_mod(source, "to_command"), expected);
    }

    #[test]
    fn inserting_before_an_attributed_module_lands_above_the_attribute() {
        // `mod bravo` sorts before the attributed `mod charlie`; the new line must
        // not split the attribute from its module.
        let source =
            "mod alpha;\n#[cfg(test)]\nmod charlie;\n\npub use alpha::*;\npub use charlie::*;\n";
        let expected = "mod alpha;\nmod bravo;\n#[cfg(test)]\nmod charlie;\n\npub use alpha::*;\npub use bravo::*;\npub use charlie::*;\n";
        assert_eq!(insert_command_mod(source, "bravo"), expected);
    }

    #[test]
    fn a_duplicate_module_is_a_no_op() {
        let source = "mod alpha;\nmod beta;\n\npub use alpha::*;\npub use beta::*;\n";
        assert_eq!(insert_command_mod(source, "beta"), source);
    }

    #[test]
    fn the_first_entry_in_an_empty_file_gets_a_section_separator() {
        // A freshly scaffolded commands/mod.rs is empty; the first command must
        // still produce the two-section mod / pub-use shape.
        assert_eq!(
            insert_command_mod("", "run"),
            "mod run;\n\npub use run::*;\n"
        );

        // A second command slots into both sections, keeping the blank.
        let after_run = insert_command_mod("", "run");
        assert_eq!(
            insert_command_mod(&after_run, "widget"),
            "mod run;\nmod widget;\n\npub use run::*;\npub use widget::*;\n"
        );
    }
}
