pub fn cargo_toml_template(command: &str, description: &str) -> String {
    format!(
        r#"[package]
name = "tyt-{command}"
version = "0.1.0"
edition = "2024"
license-file = "LICENSE"
description = "{description}"

[dependencies]
clap = {{ version = "4.5.58", features = ["derive"] }}
tyt-common = {{ version = "0.1.0" }}
tyt-injection = {{ version = "0.1.0", optional = true }}

[features]
default = ["impl"]
impl = ["dep:tyt-injection"]
"#
    )
}
