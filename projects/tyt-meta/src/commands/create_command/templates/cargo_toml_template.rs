pub fn cargo_toml_template(command: &str, description: &str) -> String {
    format!(
        r#"[package]
name = "tyt-{command}"
version = "0.1.0"
edition = "2024"
license-file = "LICENSE"
repository = "https://github.com/tyleo/tyt"
description = "{description}"
autobins = false

[[bin]]
name = "{command}"
path = "src/main.rs"
required-features = ["bin"]

[dependencies]
clap = {{ version = "4.5.58", features = ["derive"] }}
clap_complete = {{ version = "4.5", optional = true }}
tyt-common = {{ version = "0.1.0" }}
tyt-injection = {{ version = "0.1.0", optional = true }}

[features]
default = ["impl"]
impl = ["dep:tyt-injection"]
bin = ["impl", "dep:clap_complete"]
"#
    )
}
