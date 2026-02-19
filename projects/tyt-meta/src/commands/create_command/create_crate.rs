use crate::{
    Dependencies, Error, Result,
    commands::{CreateCommand, create_command},
};

pub fn create_crate(cmd: &CreateCommand, deps: &impl Dependencies) -> Result<()> {
    let command = &cmd.command;
    let name = &cmd.name;
    let description = &cmd.description;
    let snake = create_command::kebab_to_snake_case(command);
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
        &create_command::cargo_toml_template(command, description),
    )?;

    // 2. LICENSE
    deps.write(crate_dir.join("LICENSE"), create_command::LICENSE_TEMPLATE)?;

    // 3. README.md
    deps.write(
        crate_dir.join("README.md"),
        &create_command::readme_template(command, name, description),
    )?;

    // 4. src/lib.rs
    deps.write(src.join("lib.rs"), &create_command::lib_rs_template(&snake))?;

    // 5. src/main.rs
    deps.write(
        src.join("main.rs"),
        &create_command::main_rs_template(command, name),
    )?;

    // 6. src/dependencies.rs
    deps.write(
        src.join("dependencies.rs"),
        create_command::DEPENDENCIES_RS_TEMPLATE,
    )?;

    // 7. src/dependencies_impl.rs
    deps.write(
        src.join("dependencies_impl.rs"),
        create_command::DEPENDENCIES_IMPL_RS_TEMPLATE,
    )?;

    // 8. src/error.rs
    deps.write(src.join("error.rs"), create_command::ERROR_RS_TEMPLATE)?;

    // 9. src/result.rs
    deps.write(src.join("result.rs"), create_command::RESULT_RS_TEMPLATE)?;

    // 10. src/tyt_{snake}.rs
    deps.write(
        src.join(format!("tyt_{snake}.rs")),
        &create_command::tyt_enum_template_empty(name, description),
    )?;

    // 11. src/commands/mod.rs
    deps.write(commands_dir.join("mod.rs"), "")?;

    // -- Wire into existing files --

    // Workspace Cargo.toml
    create_command::wire_workspace_cargo_toml(deps, &root, command)?;

    // projects/tyt/Cargo.toml
    create_command::wire_tyt_cargo_toml(deps, &root, command)?;

    // projects/tyt/src/dependencies.rs
    create_command::wire_tyt_dependencies(deps, &root, command, name)?;

    // projects/tyt/src/dependencies_impl.rs
    create_command::wire_tyt_dependencies_impl(deps, &root, command, name)?;

    // projects/tyt/src/error.rs
    create_command::wire_tyt_error(deps, &root, command, name)?;

    // projects/tyt/src/tyt.rs
    create_command::wire_tyt_tyt_rs(deps, &root, command, name)?;

    deps.write_stdout(
        format!(
            "Created tyt-{command} crate and wired into workspace.\n\
             Next: add commands with `tyt meta create-command <Name> <command> <desc> --parent {command}`\n"
        )
        .as_bytes(),
    )?;

    Ok(())
}
