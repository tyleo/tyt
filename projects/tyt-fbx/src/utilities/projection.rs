use clap::ValueEnum;

/// Camera projection used by the `render` command.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum Projection {
    #[default]
    Perspective,
    Orthographic,
}

impl Projection {
    /// Returns the string identifier passed to Blender's `camera.type`
    /// setting.
    pub fn as_blender_type(self) -> &'static str {
        match self {
            Projection::Perspective => "PERSP",
            Projection::Orthographic => "ORTHO",
        }
    }
}
