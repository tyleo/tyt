use crate::{
    BSdfFont, BSdfMaterial, BSdfNode, BSdfObject, BSdfPattern, BSdfShades, BSdfShape2d,
    BSdfShape3d, BSdfStep,
};
use branded_id::U32Id;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// One entry of an [`SdfState`](crate::SdfState) table. The id displays as
/// `table[index]`, which points to the entry in a document.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SdfEntryId {
    /// An entry of `fonts`.
    Font(U32Id<BSdfFont>),

    /// An entry of `materials`.
    Material(U32Id<BSdfMaterial>),

    /// An entry of `nodes`.
    Node(U32Id<BSdfNode>),

    /// An entry of `objects`.
    Object(U32Id<BSdfObject>),

    /// An entry of `patterns`.
    Pattern(U32Id<BSdfPattern>),

    /// An entry of `shades`.
    Shades(U32Id<BSdfShades>),

    /// An entry of `shapes2d`.
    Shape2d(U32Id<BSdfShape2d>),

    /// An entry of `shapes3d`.
    Shape3d(U32Id<BSdfShape3d>),

    /// An entry of `steps`.
    Step(U32Id<BSdfStep>),
}

impl SdfEntryId {
    /// The name of the table holding the entry.
    pub fn table(self) -> &'static str {
        match self {
            SdfEntryId::Font(_) => "fonts",
            SdfEntryId::Material(_) => "materials",
            SdfEntryId::Node(_) => "nodes",
            SdfEntryId::Object(_) => "objects",
            SdfEntryId::Pattern(_) => "patterns",
            SdfEntryId::Shades(_) => "shades",
            SdfEntryId::Shape2d(_) => "shapes2d",
            SdfEntryId::Shape3d(_) => "shapes3d",
            SdfEntryId::Step(_) => "steps",
        }
    }
}

impl Display for SdfEntryId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let table = self.table();

        match self {
            SdfEntryId::Font(font_id) => write!(f, "{table}[{font_id}]"),
            SdfEntryId::Material(material_id) => write!(f, "{table}[{material_id}]"),
            SdfEntryId::Node(node_id) => write!(f, "{table}[{node_id}]"),
            SdfEntryId::Object(object_id) => write!(f, "{table}[{object_id}]"),
            SdfEntryId::Pattern(pattern_id) => write!(f, "{table}[{pattern_id}]"),
            SdfEntryId::Shades(shades_id) => write!(f, "{table}[{shades_id}]"),
            SdfEntryId::Shape2d(shape2d_id) => write!(f, "{table}[{shape2d_id}]"),
            SdfEntryId::Shape3d(shape3d_id) => write!(f, "{table}[{shape3d_id}]"),
            SdfEntryId::Step(step_id) => write!(f, "{table}[{step_id}]"),
        }
    }
}
