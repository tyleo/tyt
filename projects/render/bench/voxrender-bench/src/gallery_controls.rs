/// What a wearer's controllers ask of one frame of a
/// [`ModelGallery`](crate::ModelGallery). Each stick and trigger value runs
/// from -1 to 1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GalleryControls {
    /// The models to step by: 1 for the next, -1 for the one before.
    pub step: i32,

    /// Whether to switch the model between the table and life size.
    pub resize: bool,

    /// How fast to walk to the right and ahead.
    pub walk: [f32; 2],

    /// How far the turn stick leans to the right.
    pub turn: f32,

    /// How fast to rise.
    pub rise: f32,

    /// How fast the model grows. A negative value shrinks it.
    pub grow: f32,
}
