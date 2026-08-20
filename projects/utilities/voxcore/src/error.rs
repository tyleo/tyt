use crate::{
    BVoxHierarchyNode, BVoxLayer, BVoxMaterial, BVoxObject, BVoxPalette, BVoxProperty,
    BVoxValuePool, BVoxValuePoolValue, BVoxVoxel, VoxObject,
};
use branded_id::U32Id;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error from voxcore.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// A value pool was given a value outside its kind's value domain: a NaN
    /// float value or component, or an int beyond `2^53 - 1` in magnitude.
    MalformedValuePoolValue { value_id: U32Id<BVoxValuePoolValue> },

    /// An object grid of this many cells would exceed
    /// [`MAX_GRID_CELLS`](crate::VoxObject::MAX_GRID_CELLS).
    GridCellCap { cells: u64 },

    /// A mutation named an object that is not one of the state's.
    UnknownObject { object_id: U32Id<BVoxObject> },

    /// A mutation named a palette that is not one of the state's.
    UnknownPalette { palette_id: U32Id<BVoxPalette> },

    /// A mutation named a value pool that is not one of the state's.
    UnknownValuePool { value_pool_id: U32Id<BVoxValuePool> },

    /// A mutation named a hierarchy node that is not one of the state's.
    UnknownHierarchyNode { node_id: U32Id<BVoxHierarchyNode> },

    /// A mutation named a property that is not one of the palette's.
    UnknownProperty { property_id: U32Id<BVoxProperty> },

    /// A mutation named a material that is not one of the palette's.
    UnknownMaterial { material_id: U32Id<BVoxMaterial> },

    /// A mutation named a value that is not one of the value pool's.
    UnknownValuePoolValue { value_id: U32Id<BVoxValuePoolValue> },

    /// A mutation named a layer that is not one of the object's.
    UnknownLayer { layer_id: U32Id<BVoxLayer> },

    /// A mutation named a voxel outside the object's grid.
    UnknownVoxel { voxel_id: U32Id<BVoxVoxel> },

    /// A move targeted a listing position at or past the listing's count.
    IndexPastCount { index: usize, count: usize },

    /// A removal named its own removed id as the replacement.
    SelfReplacement,

    /// A reorder did not list each of the value pool's value ids exactly once.
    ValuePoolValueOrder,

    /// A voxel was given a sample count different from the layer count.
    SampleArity { samples: usize, layers: usize },

    /// A material was given a value-id count different from the property
    /// count.
    MaterialValueArity { values: usize, properties: usize },

    /// A property was given a name the palette already uses.
    DuplicatePropertyName { name: String },

    /// An inserted palette's property names a value pool that is not one of
    /// the state's.
    PropertyValuePoolRef {
        property_id: U32Id<BVoxProperty>,
        value_pool_id: U32Id<BVoxValuePool>,
    },

    /// An inserted palette's material draws a value for this property that is
    /// not one of the property's value pool's.
    MaterialValueRef {
        property_id: U32Id<BVoxProperty>,
        material_id: U32Id<BVoxMaterial>,
    },

    /// An inserted object's layer references a palette that is not one of the
    /// state's.
    LayerPaletteRef {
        layer_id: U32Id<BVoxLayer>,
        palette_id: U32Id<BVoxPalette>,
    },

    /// A live voxel's sample for this layer is not one of the layer's
    /// palette's materials.
    LayerSampleMaterial {
        layer_id: U32Id<BVoxLayer>,
        voxel_id: U32Id<BVoxVoxel>,
        material_id: U32Id<BVoxMaterial>,
    },

    /// An inserted hierarchy node, at this listing index in its batch, lists
    /// the same child node more than once.
    InsertedDuplicateChildNode {
        index: usize,
        child_id: U32Id<BVoxHierarchyNode>,
    },

    /// An inserted hierarchy node, at this listing index in its batch, places
    /// the same object more than once.
    InsertedDuplicateChildObject {
        index: usize,
        object_id: U32Id<BVoxObject>,
    },

    /// An inserted hierarchy node, at this listing index in its batch, has a
    /// non-finite transform position or scale component.
    InsertedNonFiniteTransform { index: usize },

    /// An inserted hierarchy node, at this listing index in its batch, has a
    /// zero transform scale component.
    InsertedZeroScale { index: usize },

    /// An inserted hierarchy node, at this listing index in its batch, has a
    /// transform rotation that is not a unit quaternion.
    InsertedNonUnitRotation { index: usize },

    /// An inserted batch of hierarchy nodes contains a `child_node_ids` cycle
    /// reaching the node at this listing index.
    InsertedCycle { index: usize },

    /// A value pool holds a value outside its kind's value domain.
    ValuePoolValue {
        value_pool_id: U32Id<BVoxValuePool>,
        value_id: U32Id<BVoxValuePoolValue>,
    },

    /// A palette property references a value pool that does not exist.
    PropertyValuePool {
        palette_id: U32Id<BVoxPalette>,
        property_id: U32Id<BVoxProperty>,
        value_pool_id: U32Id<BVoxValuePool>,
    },

    /// A material's value id for a property is not one of the value pool's
    /// values.
    MaterialValue {
        palette_id: U32Id<BVoxPalette>,
        property_id: U32Id<BVoxProperty>,
        material_id: U32Id<BVoxMaterial>,
    },

    /// An object references a palette that does not exist.
    PaletteRef {
        object_id: U32Id<BVoxObject>,
        palette_id: U32Id<BVoxPalette>,
    },

    /// A live voxel samples a material beyond its layer's palette.
    SampleMaterial {
        object_id: U32Id<BVoxObject>,
        voxel_id: U32Id<BVoxVoxel>,
        material_id: U32Id<BVoxMaterial>,
    },

    /// A node lists a child node that does not exist.
    ChildNode {
        node_id: U32Id<BVoxHierarchyNode>,
        child_id: U32Id<BVoxHierarchyNode>,
    },

    /// A node places an object that does not exist.
    ChildObject {
        node_id: U32Id<BVoxHierarchyNode>,
        object_id: U32Id<BVoxObject>,
    },

    /// A root references a node that does not exist.
    Root { root_id: U32Id<BVoxHierarchyNode> },

    /// The hierarchy contains a cycle reaching this node.
    Cycle { node_id: U32Id<BVoxHierarchyNode> },

    /// A node lists the same child node more than once.
    DuplicateChildNode {
        node_id: U32Id<BVoxHierarchyNode>,
        child_id: U32Id<BVoxHierarchyNode>,
    },

    /// A node places the same object more than once.
    DuplicateChildObject {
        node_id: U32Id<BVoxHierarchyNode>,
        object_id: U32Id<BVoxObject>,
    },

    /// A root lists the same node more than once.
    DuplicateRoot { root_id: U32Id<BVoxHierarchyNode> },

    /// A node's transform has a non-finite position or scale component.
    NonFiniteTransform { node_id: U32Id<BVoxHierarchyNode> },

    /// A node's transform has a zero scale component.
    ZeroScale { node_id: U32Id<BVoxHierarchyNode> },

    /// A node's transform rotation is not a unit quaternion.
    NonUnitRotation { node_id: U32Id<BVoxHierarchyNode> },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // Ids print as their bare `u32`: a branded id's `Display` carries
        // the brand name, which the surrounding wording already gives.
        match self {
            Error::MalformedValuePoolValue { value_id } => write!(
                f,
                "value {} is outside its kind's value domain",
                value_id.to_u32()
            ),
            Error::GridCellCap { cells } => write!(
                f,
                "a {cells}-cell grid exceeds the {}-cell dense cap",
                VoxObject::MAX_GRID_CELLS
            ),
            Error::UnknownObject { object_id } => {
                write!(
                    f,
                    "object {} is not one of this state's",
                    object_id.to_u32()
                )
            }
            Error::UnknownPalette { palette_id } => {
                write!(
                    f,
                    "palette {} is not one of this state's",
                    palette_id.to_u32()
                )
            }
            Error::UnknownValuePool { value_pool_id } => {
                write!(
                    f,
                    "value pool {} is not one of this state's",
                    value_pool_id.to_u32()
                )
            }
            Error::UnknownHierarchyNode { node_id } => write!(
                f,
                "hierarchy node {} is not one of this state's",
                node_id.to_u32()
            ),
            Error::UnknownProperty { property_id } => write!(
                f,
                "property {} is not one of the palette's",
                property_id.to_u32()
            ),
            Error::UnknownMaterial { material_id } => write!(
                f,
                "material {} is not one of the palette's",
                material_id.to_u32()
            ),
            Error::UnknownValuePoolValue { value_id } => {
                write!(
                    f,
                    "value {} is not one of the value pool's",
                    value_id.to_u32()
                )
            }
            Error::UnknownLayer { layer_id } => {
                write!(f, "layer {} is not one of the object's", layer_id.to_u32())
            }
            Error::UnknownVoxel { voxel_id } => {
                write!(
                    f,
                    "voxel {} is outside the object's grid",
                    voxel_id.to_u32()
                )
            }
            Error::IndexPastCount { index, count } => {
                write!(f, "index {index} is at or past the listing count {count}")
            }
            Error::SelfReplacement => {
                write!(f, "the replacement is the id being removed")
            }
            Error::ValuePoolValueOrder => write!(
                f,
                "the new order does not list each of the value pool's value ids exactly once"
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
            Error::PropertyValuePoolRef {
                property_id,
                value_pool_id,
            } => write!(
                f,
                "the inserted palette's property {} names value pool {}, which is not one of this \
                 state's",
                property_id.to_u32(),
                value_pool_id.to_u32()
            ),
            Error::MaterialValueRef {
                property_id,
                material_id,
            } => write!(
                f,
                "the inserted palette's material {} draws a value for property {} that is not one \
                 of its value pool's",
                material_id.to_u32(),
                property_id.to_u32()
            ),
            Error::LayerPaletteRef {
                layer_id,
                palette_id,
            } => write!(
                f,
                "the inserted object's layer {} references palette {}, which is not one of this \
                 state's",
                layer_id.to_u32(),
                palette_id.to_u32()
            ),
            Error::LayerSampleMaterial {
                layer_id,
                voxel_id,
                material_id,
            } => write!(
                f,
                "voxel {} samples material {} for layer {}, which is not one of the layer's \
                 palette's",
                voxel_id.to_u32(),
                material_id.to_u32(),
                layer_id.to_u32()
            ),
            Error::InsertedDuplicateChildNode { index, child_id } => write!(
                f,
                "the inserted hierarchy node at listing index {index} lists child node {} more \
                 than once",
                child_id.to_u32()
            ),
            Error::InsertedDuplicateChildObject { index, object_id } => write!(
                f,
                "the inserted hierarchy node at listing index {index} places object {} more than \
                 once",
                object_id.to_u32()
            ),
            Error::InsertedNonFiniteTransform { index } => write!(
                f,
                "the inserted hierarchy node at listing index {index} has a non-finite transform \
                 position or scale component"
            ),
            Error::InsertedZeroScale { index } => write!(
                f,
                "the inserted hierarchy node at listing index {index} has a zero transform scale \
                 component"
            ),
            Error::InsertedNonUnitRotation { index } => write!(
                f,
                "the inserted hierarchy node at listing index {index} has a transform rotation \
                 that is not a unit quaternion"
            ),
            Error::InsertedCycle { index } => write!(
                f,
                "the inserted hierarchy nodes contain a cycle reaching the node at listing index \
                 {index}"
            ),
            Error::ValuePoolValue {
                value_pool_id,
                value_id,
            } => write!(
                f,
                "value pool {} value {} is outside its kind's value domain",
                value_pool_id.to_u32(),
                value_id.to_u32()
            ),
            Error::PropertyValuePool {
                palette_id,
                property_id,
                value_pool_id,
            } => write!(
                f,
                "palette {} property {} references value pool {}, which does not exist",
                palette_id.to_u32(),
                property_id.to_u32(),
                value_pool_id.to_u32()
            ),
            Error::MaterialValue {
                palette_id,
                property_id,
                material_id,
            } => write!(
                f,
                "palette {} material {} has a value id for property {} that is not one of the value pool's values",
                palette_id.to_u32(),
                material_id.to_u32(),
                property_id.to_u32()
            ),
            Error::PaletteRef {
                object_id,
                palette_id,
            } => write!(
                f,
                "object {} references palette {}, which does not exist",
                object_id.to_u32(),
                palette_id.to_u32()
            ),
            Error::SampleMaterial {
                object_id,
                voxel_id,
                material_id,
            } => write!(
                f,
                "object {} voxel {} samples material {}, out of range of its palette",
                object_id.to_u32(),
                voxel_id.to_u32(),
                material_id.to_u32()
            ),
            Error::ChildNode { node_id, child_id } => write!(
                f,
                "hierarchy node {} lists child node {}, which does not exist",
                node_id.to_u32(),
                child_id.to_u32()
            ),
            Error::ChildObject { node_id, object_id } => write!(
                f,
                "hierarchy node {} places object {}, which does not exist",
                node_id.to_u32(),
                object_id.to_u32()
            ),
            Error::Root { root_id } => write!(
                f,
                "root references hierarchy node {}, which does not exist",
                root_id.to_u32()
            ),
            Error::Cycle { node_id } => write!(
                f,
                "hierarchy is not acyclic: a cycle reaches node {}",
                node_id.to_u32()
            ),
            Error::DuplicateChildNode { node_id, child_id } => write!(
                f,
                "hierarchy node {} lists child node {} more than once",
                node_id.to_u32(),
                child_id.to_u32()
            ),
            Error::DuplicateChildObject { node_id, object_id } => write!(
                f,
                "hierarchy node {} places object {} more than once",
                node_id.to_u32(),
                object_id.to_u32()
            ),
            Error::DuplicateRoot { root_id } => write!(
                f,
                "root lists hierarchy node {} more than once",
                root_id.to_u32()
            ),
            Error::NonFiniteTransform { node_id } => write!(
                f,
                "hierarchy node {} has a non-finite transform position or scale component",
                node_id.to_u32()
            ),
            Error::ZeroScale { node_id } => write!(
                f,
                "hierarchy node {} has a zero transform scale component",
                node_id.to_u32()
            ),
            Error::NonUnitRotation { node_id } => write!(
                f,
                "hierarchy node {} transform rotation is not a unit quaternion",
                node_id.to_u32()
            ),
        }
    }
}

impl StdError for Error {}
