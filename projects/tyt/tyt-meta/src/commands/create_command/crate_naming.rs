use crate::commands::create_command;

/// Resolved crate names for a command, with or without the `tyt-` prefix.
pub(crate) struct CrateNaming {
    prefixed: bool,
    /// Cargo package name, e.g. `tyt-vmax` or `voxl`.
    pub package: String,
    /// Library crate and root command-enum module name, e.g. `tyt_vmax` or
    /// `voxl`. The root enum lives in `src/{module}.rs`.
    pub module: String,
}

impl CrateNaming {
    /// Builds the names for `command`, applying the `tyt-` package prefix and
    /// `tyt_` module prefix when `prefix` is set.
    pub fn new(prefix: bool, command: &str) -> Self {
        let snake = create_command::kebab_to_snake_case(command);
        let (package, module) = if prefix {
            (format!("tyt-{command}"), format!("tyt_{snake}"))
        } else {
            (command.to_string(), snake)
        };
        Self {
            prefixed: prefix,
            package,
            module,
        }
    }

    /// Root command-enum name for the PascalCase `name`, e.g. `TytVMax` when
    /// prefixed or `Voxl` when not.
    pub fn root_enum(&self, name: &str) -> String {
        if self.prefixed {
            format!("Tyt{name}")
        } else {
            name.to_string()
        }
    }
}
