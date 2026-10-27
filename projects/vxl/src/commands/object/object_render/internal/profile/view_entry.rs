use crate::{
    NamedCliValue, PositiveF64,
    commands::{PoseTransformEntry, ProjectionKind},
};
use serde::Deserialize;

/// A profile's view, keyed by the name that suffixes its file.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ViewEntry {
    /// Mirrors `--view-frame`, `--view-position`, and a rotation flag, or
    /// `--view-orbit` for the whole element.
    pub(crate) transform: Option<PoseTransformEntry>,

    /// Mirrors `--view-projection`. Defaults to `perspective`.
    pub(crate) projection: Option<NamedCliValue<ProjectionKind>>,

    /// Mirrors `--view-fov`, the vertical field of view in degrees. Defaults
    /// to `35`.
    pub(crate) fov: Option<PositiveF64>,

    /// Mirrors `--view-scale`, the world units across the shorter image
    /// axis. Defaults to `fit`.
    pub(crate) scale: Option<PositiveF64>,

    /// Mirrors `--view-select`, hierarchy-path globs. Defaults to the
    /// rendered objects.
    pub(crate) select: Option<Vec<String>>,
}
