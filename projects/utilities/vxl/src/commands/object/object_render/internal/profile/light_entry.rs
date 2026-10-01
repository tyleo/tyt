use crate::{
    NamedCliValue, PositiveF64,
    commands::{
        LightKind, NonNegativeF64, PositionTransformEntry, RotationTransformEntry,
        SpotTransformEntry, SrgbColor,
    },
};
use serde::Deserialize;
use voxsmith::operations::object::RenderShadow;

/// A profile's light. Its `kind` mirrors `--light`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum LightEntry {
    Directional {
        /// Mirrors `--light-frame` and a rotation flag.
        transform: Option<RotationTransformEntry>,

        /// Mirrors `--light-shadow`.
        shadow: Option<NamedCliValue<RenderShadow>>,

        /// Mirrors `--light-color`.
        color: Option<SrgbColor>,

        /// Mirrors `--light-strength`.
        strength: Option<NonNegativeF64>,
    },

    Point {
        /// Mirrors `--light-frame` and `--light-position`, or `--light-orbit`
        /// for the whole element.
        transform: Option<PositionTransformEntry>,

        /// Mirrors `--light-shadow`.
        shadow: Option<NamedCliValue<RenderShadow>>,

        /// Mirrors `--light-color`.
        color: Option<SrgbColor>,

        /// Mirrors `--light-strength`.
        strength: Option<NonNegativeF64>,

        /// Mirrors `--light-range`, in meters. Without it, the light has no cutoff.
        range: Option<PositiveF64>,
    },

    #[serde(rename_all = "camelCase")]
    Spot {
        /// Mirrors `--light-frame`, `--light-position`, and a rotation flag,
        /// or `--light-orbit` for the whole element.
        transform: Option<SpotTransformEntry>,

        /// Mirrors `--light-shadow`.
        shadow: Option<NamedCliValue<RenderShadow>>,

        /// Mirrors `--light-color`.
        color: Option<SrgbColor>,

        /// Mirrors `--light-strength`.
        strength: Option<NonNegativeF64>,

        /// Mirrors `--light-range`, in meters. Without it, the light has no cutoff.
        range: Option<PositiveF64>,

        /// Mirrors `--light-cone`'s inner angle, in degrees. Without it, `0`.
        inner_cone: Option<NonNegativeF64>,

        /// Mirrors `--light-cone`'s outer angle, in degrees. Without it, `45`.
        outer_cone: Option<NonNegativeF64>,
    },

    Hemisphere {
        /// Mirrors `--light-sky`.
        sky: Option<SrgbColor>,

        /// Mirrors `--light-ground`.
        ground: Option<SrgbColor>,

        /// Mirrors `--light-strength`.
        strength: Option<NonNegativeF64>,
    },
}

impl LightEntry {
    /// The light's kind.
    pub(crate) fn kind(&self) -> LightKind {
        match self {
            LightEntry::Directional { .. } => LightKind::Directional,
            LightEntry::Point { .. } => LightKind::Point,
            LightEntry::Spot { .. } => LightKind::Spot,
            LightEntry::Hemisphere { .. } => LightKind::Hemisphere,
        }
    }
}
