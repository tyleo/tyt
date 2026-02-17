use crate::{
    Dependencies, Error, Result,
    commands::create_command::{
        CreateCommand, DEPENDENCIES_IMPL_RS_TEMPLATE, DEPENDENCIES_RS_TEMPLATE, ERROR_RS_TEMPLATE,
        LICENSE_TEMPLATE, RESULT_RS_TEMPLATE, cargo_toml_template, kebab_to_snake_case,
        lib_rs_template, main_rs_template, readme_template, tyt_enum_template_empty,
        wire_tyt_cargo_toml, wire_tyt_dependencies, wire_tyt_dependencies_impl, wire_tyt_error,
        wire_tyt_tyt_rs, wire_workspace_cargo_toml,
    },
};

pub fn create_crate(cmd: &CreateCommand, deps: &impl Dependencies) -> Result<()> {
    let command = &cmd.command;
    let name = &cmd.name;
    let description = &cmd.description;
    let snake = kebab_to_snake_case(command);
    let root = deps.workspace_root()?;
    let crate_dir = root.join(format!("projects/tyt-{command}"));

    if crate_dir.exists() {
        return Err(Error::Meta(format!(
            "crate directory already exists: {}",
            crate_dir.display()
        )));
    }

    let src = crate_dir.join("src");
    let commands_dir = src.join("commands");
    deps.create_dir_all(&commands_dir)?;

    // 1. Cargo.toml
    deps.write(
        crate_dir.join("Cargo.toml"),
        &cargo_toml_template(command, description),
    )?;

    // 2. LICENSE
    deps.write(crate_dir.join("LICENSE"), LICENSE_TEMPLATE)?;

    // 3. README.md
    deps.write(
        crate_dir.join("README.md"),
        &readme_template(command, name, description),
    )?;

    // 4. src/lib.rs
    deps.write(src.join("lib.rs"), &lib_rs_template(&snake))?;

    // 5. src/main.rs
    deps.write(src.join("main.rs"), &main_rs_template(command, name))?;

    // 6. src/dependencies.rs
    deps.write(src.join("dependencies.rs"), DEPENDENCIES_RS_TEMPLATE)?;

    // 7. src/dependencies_impl.rs
    deps.write(
        src.join("dependencies_impl.rs"),
        DEPENDENCIES_IMPL_RS_TEMPLATE,
    )?;

    // 8. src/error.rs
    deps.write(src.join("error.rs"), ERROR_RS_TEMPLATE)?;

    // 9. src/result.rs
    deps.write(src.join("result.rs"), RESULT_RS_TEMPLATE)?;

    // 10. src/tyt_{snake}.rs
    deps.write(
        src.join(format!("tyt_{snake}.rs")),
        &tyt_enum_template_empty(name, description),
    )?;

    // 11. src/commands/mod.rs
    deps.write(commands_dir.join("mod.rs"), "")?;

    // -- Wire into existing files --

    // Workspace Cargo.toml
    wire_workspace_cargo_toml(deps, &root, command)?;

    // projects/tyt/Cargo.toml
    wire_tyt_cargo_toml(deps, &root, command)?;

    // projects/tyt/src/dependencies.rs
    wire_tyt_dependencies(deps, &root, command, name)?;

    // projects/tyt/src/dependencies_impl.rs
    wire_tyt_dependencies_impl(deps, &root, command, name)?;

    // projects/tyt/src/error.rs
    wire_tyt_error(deps, &root, command, name)?;

    // projects/tyt/src/tyt.rs
    wire_tyt_tyt_rs(deps, &root, command, name)?;

    deps.write_stdout(
        format!(
            "Created tyt-{command} crate and wired into workspace.\n\
             Next: add commands with `tyt-meta create-command <Name> <command> <desc> --parent {command}`\n"
        )
        .as_bytes(),
    )?;

    Ok(())
}
