use crate::{BSdfMaterial, BSdfNode, BSdfShades, SdfEntryId};
use branded_id::U32Id;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// A rule an [`SdfState`](crate::SdfState) breaks.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// A name points past the end of its table.
    NameMissing {
        table: &'static str,

        name: String,

        entry_id: SdfEntryId,
    },

    /// An entry holds a NaN or an infinity.
    NonFiniteNumber { entry_id: SdfEntryId, number: f64 },

    /// An entry's reference points past the end of its table. A reference from
    /// `shapes3d`, `shapes2d`, or `nodes` into the same table also errors
    /// unless it points to an earlier entry.
    Reference {
        entry_id: SdfEntryId,

        referenced_entry_id: SdfEntryId,
    },

    /// A `json` property holds one key twice in an object.
    RepeatedMapKey {
        material_id: U32Id<BSdfMaterial>,

        key: String,
    },

    /// A names table holds one name twice.
    RepeatedName { table: &'static str, name: String },

    /// A material holds one property name twice.
    RepeatedPropertyName {
        material_id: U32Id<BSdfMaterial>,

        name: String,
    },

    /// A root node sits in another node's children.
    RootNodeChild {
        node_id: U32Id<BSdfNode>,

        parent_node_id: U32Id<BSdfNode>,
    },

    /// A root node is past the end of `nodes`.
    RootNodeMissing { node_id: U32Id<BSdfNode> },

    /// The root nodes list one node twice.
    RootNodeRepeated { node_id: U32Id<BSdfNode> },

    /// A shade's shades entry takes a base that does not come before the
    /// shade.
    ShadeBase {
        material_id: U32Id<BSdfMaterial>,

        shades_id: U32Id<BSdfShades>,

        base_id: U32Id<BSdfMaterial>,
    },

    /// A shade's index reaches its shades entry's count.
    ShadeIndex {
        material_id: U32Id<BSdfMaterial>,

        shades_id: U32Id<BSdfShades>,

        index: u32,

        count: f64,
    },

    /// A table holds more entries than a `u32` id addresses.
    TooManyEntries { table: &'static str, length: usize },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::NameMissing {
                table,
                name,
                entry_id,
            } => write!(
                f,
                "names.{table} gives `{name}` to {entry_id}, past the end of {}",
                entry_id.table()
            ),

            Error::NonFiniteNumber { entry_id, number } => {
                write!(f, "{entry_id} holds the non-finite number {number}")
            }

            Error::Reference {
                entry_id,
                referenced_entry_id,
            } if entry_id.table() == referenced_entry_id.table() => write!(
                f,
                "{entry_id} references {referenced_entry_id}, which does not come before it"
            ),

            Error::Reference {
                entry_id,
                referenced_entry_id,
            } => write!(
                f,
                "{entry_id} references {referenced_entry_id}, past the end of {}",
                referenced_entry_id.table()
            ),

            Error::RepeatedMapKey { material_id, key } => write!(
                f,
                "materials[{material_id}] holds the key `{key}` twice in one object"
            ),

            Error::RepeatedName { table, name } => {
                write!(f, "names.{table} holds `{name}` twice")
            }

            Error::RepeatedPropertyName { material_id, name } => write!(
                f,
                "materials[{material_id}] holds the property `{name}` twice"
            ),

            Error::RootNodeChild {
                node_id,
                parent_node_id,
            } => write!(
                f,
                "root node nodes[{node_id}] sits in the children of nodes[{parent_node_id}]"
            ),

            Error::RootNodeMissing { node_id } => {
                write!(f, "root node nodes[{node_id}] is past the end of nodes")
            }

            Error::RootNodeRepeated { node_id } => {
                write!(f, "the root nodes list nodes[{node_id}] twice")
            }

            Error::ShadeBase {
                material_id,
                shades_id,
                base_id,
            } => write!(
                f,
                "materials[{material_id}] takes a shade of shades[{shades_id}], whose base \
                 materials[{base_id}] does not come before it"
            ),

            Error::ShadeIndex {
                material_id,
                shades_id,
                index,
                count,
            } => write!(
                f,
                "materials[{material_id}] takes shade {index} of shades[{shades_id}], which \
                 holds {count}"
            ),

            Error::TooManyEntries { table, length } => write!(
                f,
                "{table} holds {length} entries, more than a u32 id addresses"
            ),
        }
    }
}

impl StdError for Error {}
