use clap::ValueEnum;

/// Render engine used by the `render` command.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Renderer {
    /// Blender's real-time rasterizer. Fast; suitable for previews.
    Eevee,
    /// Blender's path tracer. Slow; photorealistic output.
    Cycles,
}

impl Renderer {
    /// Returns the string identifier passed to Blender's
    /// `scene.render.engine` setting.
    pub fn as_blender_engine(self) -> &'static str {
        match self {
            Renderer::Eevee => "BLENDER_EEVEE",
            Renderer::Cycles => "CYCLES",
        }
    }
}
