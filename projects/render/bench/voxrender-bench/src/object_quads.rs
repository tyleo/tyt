use ty_math::TyVector3F32;

/// One object's opaque quads, as a ray query's structures take them.
#[derive(Default)]
pub struct ObjectQuads {
    /// The quads' corners in the object's cells.
    pub positions: Vec<TyVector3F32>,

    /// Two triangles a quad, as indices into
    /// [`positions`](Self::positions).
    pub indices: Vec<u32>,

    /// Each quad's face: its axis doubled, plus one where the face looks up
    /// the axis.
    pub faces: Vec<u32>,

    /// Each quad's material, as its place among its scene's.
    pub materials: Vec<u32>,
}
