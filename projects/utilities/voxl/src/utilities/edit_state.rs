use clap::ValueEnum;

/// When `to-voxj` records each object's editor build volume in the document's
/// `edit_state`. The build volume is the working grid an object was authored in,
/// which can be larger than the tight grid its live voxels occupy.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum EditState {
    /// Record it only when some object carries margin around its live voxels, so
    /// the build volume is kept without spending space when it is redundant.
    #[default]
    #[value(name = "auto")]
    Auto,
    /// Always record it, even when every object is already tight.
    #[value(name = "true")]
    True,
    /// Never record it. Margin around an object's live voxels is lost on reload.
    #[value(name = "false")]
    False,
}
