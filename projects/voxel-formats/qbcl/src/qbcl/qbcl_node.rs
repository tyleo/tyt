use crate::qbcl::QbclNodeBody;

/// One node of a `.qbcl` scene tree: a name, the editor flags every node carries,
/// and a type-specific [`body`](Self::body).
#[derive(Clone, Debug, PartialEq)]
pub struct QbclNode {
    /// Node name.
    pub name: String,

    /// Whether the node is shown in the editor (the first node flag byte).
    pub visible: bool,

    /// Whether the node is locked in the editor (the third node flag byte).
    pub locked: bool,

    /// The node body and type.
    pub body: QbclNodeBody,
}

impl Default for QbclNode {
    fn default() -> Self {
        Self {
            name: String::new(),
            visible: true,
            locked: false,
            body: QbclNodeBody::default(),
        }
    }
}
