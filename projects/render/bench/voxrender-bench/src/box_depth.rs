/// The depth a mask march's fragments leave in the depth buffer.
#[derive(Clone, Copy, Debug)]
pub enum BoxDepth {
    /// Each fragment writes its hit's depth. Quest's depth test then runs
    /// after the march, so every box a ray crosses marches.
    Hit,

    /// Each fragment keeps the depth of its box's face and discards a miss.
    /// The boxes draw nearest first, and the depth test rejects a fragment
    /// behind a nearer hit before it marches. Two boxes that share no space
    /// order their hits as they order their faces. A box that crosses
    /// another placement's box writes its hit's depth.
    Face,
}

impl BoxDepth {
    /// The depth's name in a report.
    pub fn label(self) -> &'static str {
        match self {
            BoxDepth::Hit => "hit depth",
            BoxDepth::Face => "face depth",
        }
    }
}
