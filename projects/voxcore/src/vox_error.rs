use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

/// An error from voxcore: a [`VoxState`](crate::VoxState) whose cross-references
/// do not resolve or whose hierarchy has a cycle (see
/// [`validate`](crate::VoxState::validate)). Ids are reported as their `u32`
/// listing index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VoxError {
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
}

impl Display for VoxError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            VoxError::PaletteRef { object, palette } => write!(
                f,
                "object {object} references palette {palette}, which does not exist"
            ),
            VoxError::SampleCell {
                object,
                voxel,
                cell,
            } => write!(
                f,
                "object {object} voxel {voxel} samples cell {cell}, out of range of its palette"
            ),
            VoxError::ChildNode { node, child } => write!(
                f,
                "hierarchy node {node} lists child node {child}, which does not exist"
            ),
            VoxError::ChildObject { node, object } => write!(
                f,
                "hierarchy node {node} places object {object}, which does not exist"
            ),
            VoxError::Root { root } => {
                write!(
                    f,
                    "root references hierarchy node {root}, which does not exist"
                )
            }
            VoxError::Cycle { node } => {
                write!(f, "hierarchy is not acyclic: a cycle reaches node {node}")
            }
        }
    }
}

impl Error for VoxError {}
