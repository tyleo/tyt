/// A palette edit's write of one expression's value into one property.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyWrite {
    /// The property written, added when the palette lacks it.
    pub property: String,

    /// Evaluated in the scope at the program's end.
    pub expression: String,
}
