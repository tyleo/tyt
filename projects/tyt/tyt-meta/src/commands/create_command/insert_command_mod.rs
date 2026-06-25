pub fn insert_command_mod(contents: &str, command_snake: &str) -> String {
    let mod_line = format!("mod {command_snake};");
    let use_line = format!("pub use {command_snake}::*;");

    let lines: Vec<&str> = contents.lines().collect();

    // Split into mod block and use block
    let mut mod_lines: Vec<String> = Vec::new();
    let mut use_lines: Vec<String> = Vec::new();

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("mod ") {
            mod_lines.push(line.to_string());
        } else if trimmed.starts_with("pub use ") {
            use_lines.push(line.to_string());
        }
    }

    mod_lines.push(mod_line);
    mod_lines.sort();
    use_lines.push(use_line);
    use_lines.sort();

    let mut result = mod_lines.join("\n");
    result.push_str("\n\n");
    result.push_str(&use_lines.join("\n"));
    result.push('\n');
    result
}
