use crate::{
    BVoxHierarchyNode, BVoxLayer, BVoxMaterial, BVoxObject, BVoxPalette, BVoxPoolValue,
    BVoxProperty, BVoxValuePool, BVoxVoxel, VoxObject,
};
use branded_id::U32Id;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error from voxcore.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// A value pool has no values.
    EmptyPool { pool: U32Id<BVoxValuePool> },

    /// A value pool was given no values at construction.
    EmptyPoolValues,

    /// A value pool was given malformed `min`/`max` bounds at construction:
    /// non-finite, not integer-valued for an `int` pool, or `min` greater
    /// than `max`.
    MalformedPoolBound,

    /// A value pool was given a value that is malformed for its kind or
    /// outside its bounds.
    MalformedPoolValue { value: U32Id<BVoxPoolValue> },

    /// An object grid of this many cells would exceed
    /// [`MAX_GRID_CELLS`](crate::VoxObject::MAX_GRID_CELLS).
    GridCellCap { cells: u64 },

    /// A mutation named an object that is not one of the state's.
    UnknownObject { object: U32Id<BVoxObject> },

    /// A mutation named a palette that is not one of the state's.
    UnknownPalette { palette: U32Id<BVoxPalette> },

    /// A mutation named a value pool that is not one of the state's.
    UnknownValuePool { pool: U32Id<BVoxValuePool> },

    /// A mutation named a hierarchy node that is not one of the state's.
    UnknownHierarchyNode { node: U32Id<BVoxHierarchyNode> },

    /// A mutation named a property that is not one of the palette's.
    UnknownProperty { property: U32Id<BVoxProperty> },

    /// A mutation named a material that is not one of the palette's.
    UnknownMaterial { material: U32Id<BVoxMaterial> },

    /// A mutation named a value that is not one of the pool's.
    UnknownPoolValue { value: U32Id<BVoxPoolValue> },

    /// A mutation named a layer that is not one of the object's.
    UnknownLayer { layer: U32Id<BVoxLayer> },

    /// A mutation named a voxel outside the object's grid.
    UnknownVoxel { voxel: U32Id<BVoxVoxel> },

    /// A move targeted a listing position at or past the listing's count.
    IndexPastCount { index: usize, count: usize },

    /// A removal named its own removed id as the replacement.
    SelfReplacement,

    /// A reorder did not list each of the pool's value ids exactly once.
    PoolValueOrder,

    /// A voxel was given a sample count different from the layer count.
    SampleArity { samples: usize, layers: usize },

    /// A material was given a value-id count different from the property
    /// count.
    MaterialValueArity { values: usize, properties: usize },

    /// A property was given a name the palette already uses.
    DuplicatePropertyName { name: String },

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
            Error::EmptyPoolValues => {
                write!(f, "a value pool needs at least one value")
            }
            Error::MalformedPoolBound => {
                write!(f, "the value pool's min/max bounds are malformed")
            }
            Error::MalformedPoolValue { value } => write!(
                f,
                "value {} is malformed for its kind or out of bounds",
                value.to_u32()
            ),
            Error::GridCellCap { cells } => write!(
                f,
                "a {cells}-cell grid exceeds the {}-cell dense cap",
                VoxObject::MAX_GRID_CELLS
            ),
            Error::UnknownObject { object } => {
                write!(f, "object {} is not one of this state's", object.to_u32())
            }
            Error::UnknownPalette { palette } => {
                write!(f, "palette {} is not one of this state's", palette.to_u32())
            }
            Error::UnknownValuePool { pool } => {
                write!(f, "value pool {} is not one of this state's", pool.to_u32())
            }
            Error::UnknownHierarchyNode { node } => write!(
                f,
                "hierarchy node {} is not one of this state's",
                node.to_u32()
            ),
            Error::UnknownProperty { property } => write!(
                f,
                "property {} is not one of the palette's",
                property.to_u32()
            ),
            Error::UnknownMaterial { material } => write!(
                f,
                "material {} is not one of the palette's",
                material.to_u32()
            ),
            Error::UnknownPoolValue { value } => {
                write!(f, "value {} is not one of the pool's", value.to_u32())
            }
            Error::UnknownLayer { layer } => {
                write!(f, "layer {} is not one of the object's", layer.to_u32())
            }
            Error::UnknownVoxel { voxel } => {
                write!(f, "voxel {} is outside the object's grid", voxel.to_u32())
            }
            Error::IndexPastCount { index, count } => {
                write!(f, "index {index} is at or past the listing count {count}")
            }
            Error::SelfReplacement => {
                write!(f, "the replacement is the id being removed")
            }
            Error::PoolValueOrder => write!(
                f,
                "the new order does not list each of the pool's value ids exactly once"
            ),
            Error::SampleArity { samples, layers } => {
                write!(f, "{samples} samples were given for {layers} layers")
            }
            Error::MaterialValueArity { values, properties } => write!(
                f,
                "{values} value ids were given for {properties} properties"
            ),
            Error::DuplicatePropertyName { name } => {
                write!(f, "a property named \"{name}\" already exists")
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
