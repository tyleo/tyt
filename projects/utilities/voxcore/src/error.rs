use crate::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxPoolValue, BVoxProperty,
    BVoxValuePool, BVoxVoxel,
};
use branded_id::U32Id;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error from voxcore: a [`VoxMain`](crate::VoxMain) that violates a rule
/// [`validate`](crate::VoxMain::validate) checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// A value pool has no values.
    EmptyPool { pool: U32Id<BVoxValuePool> },

    /// A value pool's `min`/`max` bounds are malformed: non-finite, not
    /// integer-valued for an `int` pool, or `min` greater than `max`.
    PoolBound { pool: U32Id<BVoxValuePool> },

    /// A value pool holds a value that is malformed for its kind or outside its
    /// bounds.
    PoolValue {
        pool: U32Id<BVoxValuePool>,
        value: U32Id<BVoxPoolValue>,
    },

    /// A palette property references a value pool that does not exist.
    PropertyPool {
        palette: U32Id<BVoxPalette>,
        property: U32Id<BVoxProperty>,
        pool: U32Id<BVoxValuePool>,
    },

    /// A material's value id for a property is not one of the pool's values.
    MaterialValue {
        palette: U32Id<BVoxPalette>,
        property: U32Id<BVoxProperty>,
        material: U32Id<BVoxMaterial>,
    },

    /// A palette has no materials.
    PaletteWithoutMaterials { palette: U32Id<BVoxPalette> },

    /// An object references a palette that does not exist.
    PaletteRef {
        object: U32Id<BVoxObject>,
        palette: U32Id<BVoxPalette>,
    },

    /// A live voxel samples a material beyond its layer's palette.
    SampleMaterial {
        object: U32Id<BVoxObject>,
        voxel: U32Id<BVoxVoxel>,
        material: U32Id<BVoxMaterial>,
    },

    /// A node lists a child node that does not exist.
    ChildNode {
        node: U32Id<BVoxHierarchyNode>,
        child: U32Id<BVoxHierarchyNode>,
    },

    /// A node places an object that does not exist.
    ChildObject {
        node: U32Id<BVoxHierarchyNode>,
        object: U32Id<BVoxObject>,
    },

    /// A root references a node that does not exist.
    Root { root: U32Id<BVoxHierarchyNode> },

    /// The hierarchy contains a cycle reaching this node.
    Cycle { node: U32Id<BVoxHierarchyNode> },

    /// A node lists the same child node more than once.
    DuplicateChildNode {
        node: U32Id<BVoxHierarchyNode>,
        child: U32Id<BVoxHierarchyNode>,
    },

    /// A node places the same object more than once.
    DuplicateChildObject {
        node: U32Id<BVoxHierarchyNode>,
        object: U32Id<BVoxObject>,
    },

    /// A root lists the same node more than once.
    DuplicateRoot { root: U32Id<BVoxHierarchyNode> },

    /// A node's transform has a non-finite position or scale component.
    NonFiniteTransform { node: U32Id<BVoxHierarchyNode> },

    /// A node's transform has a zero scale component.
    ZeroScale { node: U32Id<BVoxHierarchyNode> },

    /// A node's transform rotation is not a unit quaternion.
    NonUnitRotation { node: U32Id<BVoxHierarchyNode> },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // Ids print as their bare `u32`: a branded id's own `Display` carries
        // the brand name, which the surrounding wording already gives.
        match self {
            Error::EmptyPool { pool } => {
                write!(f, "value pool {} has no values", pool.to_u32())
            }
            Error::PoolBound { pool } => {
                write!(
                    f,
                    "value pool {} has malformed min/max bounds",
                    pool.to_u32()
                )
            }
            Error::PoolValue { pool, value } => write!(
                f,
                "value pool {} value {} is malformed for its kind or out of bounds",
                pool.to_u32(),
                value.to_u32()
            ),
            Error::PropertyPool {
                palette,
                property,
                pool,
            } => write!(
                f,
                "palette {} property {} references value pool {}, which does not exist",
                palette.to_u32(),
                property.to_u32(),
                pool.to_u32()
            ),
            Error::MaterialValue {
                palette,
                property,
                material,
            } => write!(
                f,
                "palette {} material {} has a value id for property {} that is not one of the pool's values",
                palette.to_u32(),
                material.to_u32(),
                property.to_u32()
            ),
            Error::PaletteWithoutMaterials { palette } => {
                write!(f, "palette {} has no materials", palette.to_u32())
            }
            Error::PaletteRef { object, palette } => write!(
                f,
                "object {} references palette {}, which does not exist",
                object.to_u32(),
                palette.to_u32()
            ),
            Error::SampleMaterial {
                object,
                voxel,
                material,
            } => write!(
                f,
                "object {} voxel {} samples material {}, out of range of its palette",
                object.to_u32(),
                voxel.to_u32(),
                material.to_u32()
            ),
            Error::ChildNode { node, child } => write!(
                f,
                "hierarchy node {} lists child node {}, which does not exist",
                node.to_u32(),
                child.to_u32()
            ),
            Error::ChildObject { node, object } => write!(
                f,
                "hierarchy node {} places object {}, which does not exist",
                node.to_u32(),
                object.to_u32()
            ),
            Error::Root { root } => write!(
                f,
                "root references hierarchy node {}, which does not exist",
                root.to_u32()
            ),
            Error::Cycle { node } => write!(
                f,
                "hierarchy is not acyclic: a cycle reaches node {}",
                node.to_u32()
            ),
            Error::DuplicateChildNode { node, child } => write!(
                f,
                "hierarchy node {} lists child node {} more than once",
                node.to_u32(),
                child.to_u32()
            ),
            Error::DuplicateChildObject { node, object } => write!(
                f,
                "hierarchy node {} places object {} more than once",
                node.to_u32(),
                object.to_u32()
            ),
            Error::DuplicateRoot { root } => write!(
                f,
                "root lists hierarchy node {} more than once",
                root.to_u32()
            ),
            Error::NonFiniteTransform { node } => write!(
                f,
                "hierarchy node {} has a non-finite transform position or scale component",
                node.to_u32()
            ),
            Error::ZeroScale { node } => write!(
                f,
                "hierarchy node {} has a zero transform scale component",
                node.to_u32()
            ),
            Error::NonUnitRotation { node } => write!(
                f,
                "hierarchy node {} transform rotation is not a unit quaternion",
                node.to_u32()
            ),
        }
    }
}

impl StdError for Error {}
