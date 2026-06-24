use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error from voxcore: a [`VoxState`](crate::VoxState) whose cross-references
/// do not resolve, whose hierarchy has a cycle, or whose roots, palette refs, or
/// node children repeat an id (see [`validate`](crate::VoxState::validate)). Ids
/// are reported as their `u32` listing index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// An object references a palette that does not exist.
    PaletteRef { object: u32, palette: u32 },

    /// A live voxel samples a cell beyond its palette's cells.
    SampleCell { object: u32, voxel: u32, cell: u32 },

    /// A node lists a child node that does not exist.
    ChildNode { node: u32, child: u32 },

    /// A node places an object that does not exist.
    ChildObject { node: u32, object: u32 },

    /// A root references a node that does not exist.
    Root { root: u32 },

    /// The hierarchy contains a cycle reaching this node.
    Cycle { node: u32 },

    /// An object references the same palette through more than one reference.
    DuplicatePaletteRef { object: u32, palette: u32 },

    /// A node lists the same child node more than once.
    DuplicateChildNode { node: u32, child: u32 },

    /// A node places the same object more than once.
    DuplicateChildObject { node: u32, object: u32 },

    /// A root lists the same node more than once.
    DuplicateRoot { root: u32 },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::PaletteRef { object, palette } => write!(
                f,
                "object {object} references palette {palette}, which does not exist"
            ),
            Error::SampleCell {
                object,
                voxel,
                cell,
            } => write!(
                f,
                "object {object} voxel {voxel} samples cell {cell}, out of range of its palette"
            ),
            Error::ChildNode { node, child } => write!(
                f,
                "hierarchy node {node} lists child node {child}, which does not exist"
            ),
            Error::ChildObject { node, object } => write!(
                f,
                "hierarchy node {node} places object {object}, which does not exist"
            ),
            Error::Root { root } => {
                write!(
                    f,
                    "root references hierarchy node {root}, which does not exist"
                )
            }
            Error::Cycle { node } => {
                write!(f, "hierarchy is not acyclic: a cycle reaches node {node}")
            }
            Error::DuplicatePaletteRef { object, palette } => write!(
                f,
                "object {object} references palette {palette} more than once"
            ),
            Error::DuplicateChildNode { node, child } => write!(
                f,
                "hierarchy node {node} lists child node {child} more than once"
            ),
            Error::DuplicateChildObject { node, object } => write!(
                f,
                "hierarchy node {node} places object {object} more than once"
            ),
            Error::DuplicateRoot { root } => {
                write!(f, "root lists hierarchy node {root} more than once")
            }
        }
    }
}

impl StdError for Error {}
