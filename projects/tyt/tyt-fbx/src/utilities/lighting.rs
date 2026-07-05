use clap::ValueEnum;

/// Lighting preset used by the `render` command.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Lighting {
    /// Even world-background illumination so every face reads clearly. No
    /// directional shadows.
    Environment,
    /// Key / fill / rim area lights. Strong contrast.
    ThreePoint,
    /// Softer three-point variant with reduced energy for flatter tonal range.
    Studio,
    /// Single camera-aligned sun light for even, low-contrast illumination.
    Flat,
    /// Do not add any lights. Relies on whatever lights (if any) exist in the
    /// imported FBX.
    None,
}

impl Lighting {
    /// Returns the string identifier passed to the Blender render script.
    pub fn as_blender_mode(self) -> &'static str {
        match self {
            Lighting::Environment => "environment",
            Lighting::ThreePoint => "three-point",
            Lighting::Studio => "studio",
            Lighting::Flat => "flat",
            Lighting::None => "none",
        }
    }
}
