pub fn readme_template(package: &str, name: &str, description: &str) -> String {
    format!("# {package} - {name}\n\n{description}\n")
}
