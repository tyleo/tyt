use crate::CliValue;
use voxconv::voxj::EditStateMode;

impl CliValue for EditStateMode {
    const VARIANTS: &'static [Self] = &[
        EditStateMode::Auto,
        EditStateMode::Always,
        EditStateMode::Never,
    ];

    fn name(self) -> &'static str {
        match self {
            EditStateMode::Auto => "auto",
            EditStateMode::Always => "true",
            EditStateMode::Never => "false",
        }
    }

    fn help(self) -> &'static str {
        match self {
            EditStateMode::Auto => {
                "Record it only when some object carries margin around its live voxels"
            }
            EditStateMode::Always => "Always record it, even when every object is already tight",
            EditStateMode::Never => {
                "Never record it; margin around an object's live voxels is lost on reload"
            }
        }
    }
}
