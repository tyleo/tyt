use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    path::PathBuf,
};

/// Where a profile of the set comes from: the binary or a `.vxlconfig`.
#[derive(Clone, Debug, PartialEq)]
pub enum ProfileOrigin {
    /// Embedded in the binary.
    BuiltIn,

    /// Read from the `.vxlconfig` at the path.
    File(PathBuf),
}

impl Display for ProfileOrigin {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            ProfileOrigin::BuiltIn => formatter.write_str("built in"),
            ProfileOrigin::File(path) => path.display().fmt(formatter),
        }
    }
}
