use sdfj::SdfjFile;
use std::path::PathBuf;

/// A library as the cascade defines it, with a file's path resolved.
#[derive(Clone, Debug, PartialEq)]
pub enum SdfDocLibrary {
    /// A document a layer holds.
    Embedded(Box<SdfjFile>),

    /// An `.sdfj` or `.sdfjz` file at a path resolved against its `.vxlconfig`.
    File(PathBuf),
}
