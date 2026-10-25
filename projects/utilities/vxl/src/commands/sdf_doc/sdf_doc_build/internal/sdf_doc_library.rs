use sdfj::SdfjFile;
use std::path::PathBuf;

/// A library as the cascade defines it, with a file's path resolved.
#[derive(Clone, Debug, PartialEq)]
pub enum SdfDocLibrary {
    /// A document a layer holds, with the layer's directory. vxl's built-in
    /// layer has none.
    Embedded {
        document: Box<SdfjFile>,

        dir: Option<PathBuf>,
    },

    /// An `.sdfj` or `.sdfjz` file at a path resolved against its `.vxlconfig`.
    File(PathBuf),
}
