use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error from voxcore: a [`VoxMain`](crate::VoxMain) that violates a rule
/// [`validate`](crate::VoxMain::validate) checks. Ids are reported as their
/// `u32` listing index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// A value pool has no values.
    EmptyPool { pool: u32 },

    /// A value pool's `min`/`max` bounds are malformed: non-finite, not
    /// integer-valued for an `int` pool, or `min` greater than `max`.
    PoolBound { pool: u32 },

    /// A value pool holds a value that is malformed for its kind or outside its
    /// bounds, at the given value id.
    PoolValue { pool: u32, index: u32 },

    /// A palette property references a value pool that does not exist.
    PropertyPool {
        palette: u32,
        property: u32,
        pool: u32,
    },

    /// A palette declares the same name on more than one property.
    DuplicatePropertyName { palette: u32, property: u32 },

    /// A material's value id for a property is beyond the pool's
    /// values.
    MaterialValue {
        palette: u32,
        property: u32,
        material: u32,
    },

    /// A palette has no materials.
    PaletteWithoutMaterials { palette: u32 },

    /// An object references a palette that does not exist.
    PaletteRef { object: u32, palette: u32 },

    /// A live voxel samples a material beyond its layer's palette.
    SampleMaterial {
        object: u32,
        voxel: u32,
        material: u32,
    },

    /// A node lists a child node that does not exist.
    ChildNode { node: u32, child: u32 },

    /// A node places an object that does not exist.
    ChildObject { node: u32, object: u32 },

    /// A root references a node that does not exist.
    Root { root: u32 },

    /// The hierarchy contains a cycle reaching this node.
    Cycle { node: u32 },

    /// A node lists the same child node more than once.
    DuplicateChildNode { node: u32, child: u32 },

    /// A node places the same object more than once.
    DuplicateChildObject { node: u32, object: u32 },

    /// A root lists the same node more than once.
    DuplicateRoot { root: u32 },

    /// A node's transform has a non-finite position or scale component.
    NonFiniteTransform { node: u32 },

    /// A node's transform has a zero scale component.
    ZeroScale { node: u32 },

    /// A node's transform rotation is not a unit quaternion.
    NonUnitRotation { node: u32 },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::EmptyPool { pool } => write!(f, "value pool {pool} has no values"),
            Error::PoolBound { pool } => {
                write!(f, "value pool {pool} has malformed min/max bounds")
            }
            Error::PoolValue { pool, index } => write!(
                f,
                "value pool {pool} value {index} is malformed for its kind or out of bounds"
            ),
            Error::PropertyPool {
                palette,
                property,
                pool,
            } => write!(
                f,
                "palette {palette} property {property} references value pool {pool}, which does not exist"
            ),
            Error::DuplicatePropertyName { palette, property } => write!(
                f,
                "palette {palette} property {property} duplicates another property's name"
            ),
            Error::MaterialValue {
                palette,
                property,
                material,
            } => write!(
                f,
                "palette {palette} material {material} has a value id for property {property} out of the pool's range"
            ),
            Error::PaletteWithoutMaterials { palette } => {
                write!(f, "palette {palette} has no materials")
            }
            Error::PaletteRef { object, palette } => write!(
                f,
                "object {object} references palette {palette}, which does not exist"
            ),
            Error::SampleMaterial {
                object,
                voxel,
                material,
            } => write!(
                f,
                "object {object} voxel {voxel} samples material {material}, out of range of its palette"
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
            Error::NonFiniteTransform { node } => write!(
                f,
                "hierarchy node {node} has a non-finite transform position or scale component"
            ),
            Error::ZeroScale { node } => write!(
                f,
                "hierarchy node {node} has a zero transform scale component"
            ),
            Error::NonUnitRotation { node } => write!(
                f,
                "hierarchy node {node} transform rotation is not a unit quaternion"
            ),
        }
    }
}

impl StdError for Error {}
