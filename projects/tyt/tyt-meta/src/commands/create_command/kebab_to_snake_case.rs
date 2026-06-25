pub fn kebab_to_snake_case(command: &str) -> String {
    command.replace('-', "_")
}
