/// Converts a kebab-case command name to PascalCase (e.g., `foo-bar` → `FooBar`).
pub fn kebab_to_pascal_case(command: &str) -> String {
    command
        .split('-')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect()
}
