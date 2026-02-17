pub fn readme_template(command: &str, name: &str, description: &str) -> String {
    format!("# tyt-{command} - {name}\n\n{description}\n")
}
