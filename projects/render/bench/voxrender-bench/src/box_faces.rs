/// Which faces of its boxes a box raster draws.
#[derive(Clone, Copy, Debug)]
pub enum BoxFaces {
    /// The back faces, in placement order. Each fragment marches before the
    /// depth test.
    Back,

    /// The front faces, nearest box first. Each fragment declares that its
    /// hit lies no nearer than its face, so the depth test can run before
    /// the march. A box that a ray can start inside draws its back faces.
    Front,
}

impl BoxFaces {
    /// The faces' name in a report.
    pub fn label(self) -> &'static str {
        match self {
            BoxFaces::Back => "back",
            BoxFaces::Front => "front",
        }
    }
}
