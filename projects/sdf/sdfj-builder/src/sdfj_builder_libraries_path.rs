/// The path in the builder's directory where a run writes the libraries the
/// model reads. The file holds a JSON array with each library's `name` and
/// `.sdfj` `document` in the build's list order.
pub const SDFJ_BUILDER_LIBRARIES_PATH: &str = "ts/libraries.json";
