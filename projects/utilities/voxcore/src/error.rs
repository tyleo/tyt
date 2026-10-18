use crate::{
    BVoxHierarchyNode, BVoxLayer, BVoxMaterial, BVoxObject, BVoxPalette, BVoxProperty,
    BVoxValuePool, BVoxValuePoolValue, BVoxVoxel, VoxObject,
};
use branded_id::{SliceExt, U32Id};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use ty_math::{TyVector3I32, TyVector3U32};

/// An error from voxcore.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The ext could not follow a mutation. A `will` hook's refusal leaves the
    /// state unchanged.
    Ext { reason: String },

    /// A value pool was given a value outside its kind's value domain.
    MalformedValuePoolValue { value_id: U32Id<BVoxValuePoolValue> },

    /// A value added to or set in a value pool lies outside its kind's value
    /// domain.
    MalformedRetainedValue,

    /// A value added to or set in a value pool differs from the value pool's
    /// kind.
    RetainedValueKind,

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

    /// A voxel remap moved this live voxel to `position`, outside the new
    /// grid of `bounds`.
    RemappedVoxelOutsideGrid {
        voxel_id: U32Id<BVoxVoxel>,

        position: TyVector3I32,

        bounds: TyVector3U32,
    },

    /// A voxel remap moved two live voxels onto one cell.
    RemappedVoxelCollision {
        voxel_id: U32Id<BVoxVoxel>,

        other_voxel_id: U32Id<BVoxVoxel>,
    },

    /// A voxel resample drew a cell from `position`, outside the old grid of
    /// `bounds`.
    ResampleSourceOutsideGrid {
        position: TyVector3U32,

        bounds: TyVector3U32,
    },

    /// A color read resolved `baseColor` to a property whose value pool holds
    /// no colors.
    NonColorProperty {
        palette_id: U32Id<BVoxPalette>,

        property_id: U32Id<BVoxProperty>,
    },

    /// A move targeted a listing position at or past the listing's count.
    IndexPastCount { index: usize, count: usize },

    /// A release named a material a live voxel still samples; `object_ids`
    /// lists the sampling objects, in listing order.
    MaterialInUse {
        material_id: U32Id<BVoxMaterial>,

        object_ids: Vec<U32Id<BVoxObject>>,
    },

    /// A release named a value a palette cell still draws; `palette_ids` lists
    /// the drawing palettes, in listing order.
    ValuePoolValueInUse {
        value_id: U32Id<BVoxValuePoolValue>,

        palette_ids: Vec<U32Id<BVoxPalette>>,
    },

    /// A release named a palette an object layer still references; `object_ids`
    /// lists the referencing objects, in listing order.
    PaletteInUse {
        palette_id: U32Id<BVoxPalette>,

        object_ids: Vec<U32Id<BVoxObject>>,
    },

    /// A release named a value pool a palette property still references;
    /// `palette_ids` lists the referencing palettes, in listing order.
    ValuePoolInUse {
        value_pool_id: U32Id<BVoxValuePool>,

        palette_ids: Vec<U32Id<BVoxPalette>>,
    },

    /// A release named an object a hierarchy node still places; `node_ids`
    /// lists the placing nodes, in listing order.
    ObjectInUse {
        object_id: U32Id<BVoxObject>,

        node_ids: Vec<U32Id<BVoxHierarchyNode>>,
    },

    /// A release named a hierarchy node still referenced: `parent_ids` lists
    /// its parents, in listing order, and `root` reports whether the roots list
    /// it.
    HierarchyNodeInUse {
        node_id: U32Id<BVoxHierarchyNode>,

        parent_ids: Vec<U32Id<BVoxHierarchyNode>>,

        root: bool,
    },

    /// A reorder did not list each of the value pool's value ids exactly once.
    ValuePoolValueOrder,

    /// A voxel was given a sample count different from the layer count.
    SampleArity { samples: usize, layers: usize },

    /// A material was given a value-id count different from the property count.
    MaterialValueArity { values: usize, properties: usize },

    /// A property was given a name the palette already uses.
    DuplicatePropertyName { name: String },

    /// A property retained without a default value would leave the palette's
    /// materials without a value for it.
    PropertyWithoutDefault { materials: usize },

    /// A layer retained without a default material would leave the object's
    /// live voxels without a sample in it.
    LayerWithoutDefault { live_voxels: usize },

    /// An inserted palette's property names a value pool that is not one of the
    /// state's.
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

    /// A live voxel's sample for this layer is not one of the layer's palette's
    /// materials.
    LayerSampleMaterial {
        layer_id: U32Id<BVoxLayer>,

        voxel_id: U32Id<BVoxVoxel>,

        material_id: U32Id<BVoxMaterial>,
    },

    /// An inserted or replaced hierarchy node, at this listing index in its
    /// batch, lists the same child node more than once.
    InsertedDuplicateChildNode {
        index: usize,

        child_id: U32Id<BVoxHierarchyNode>,
    },

    /// An inserted or replaced hierarchy node, at this listing index in its
    /// batch, places the same object more than once.
    InsertedDuplicateChildObject {
        index: usize,

        object_id: U32Id<BVoxObject>,
    },

    /// An inserted or replaced hierarchy node, at this listing index in its
    /// batch, has a non-finite transform position or scale component.
    InsertedNonFiniteTransform { index: usize },

    /// An inserted or replaced hierarchy node, at this listing index in its
    /// batch, has a zero transform scale component.
    InsertedZeroScale { index: usize },

    /// An inserted or replaced hierarchy node, at this listing index in its
    /// batch, has a transform rotation that is not a unit quaternion.
    InsertedNonUnitRotation { index: usize },

    /// The `child_node_ids` of an inserted or replaced batch of hierarchy
    /// nodes form a cycle reaching the node at this listing index in the batch.
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
        // Ids print as their bare `u32`: a branded id's `Display` carries the
        // brand name, which the surrounding wording already gives.
        match self {
            Error::Ext { reason } => write!(f, "ext: {reason}"),

            Error::MalformedValuePoolValue { value_id } => {
                write!(f, "value {value_id} is outside its kind's value domain")
            }

            Error::MalformedRetainedValue => {
                write!(f, "the new value is outside its kind's value domain")
            }

            Error::RetainedValueKind => {
                write!(f, "the new value's kind differs from the value pool's")
            }

            Error::GridCellCap { cells } => write!(
                f,
                "a {cells}-cell grid exceeds the {}-cell dense cap",
                VoxObject::MAX_GRID_CELLS
            ),

            Error::UnknownObject { object_id } => {
                write!(f, "object {object_id} is not one of this state's")
            }

            Error::UnknownPalette { palette_id } => {
                write!(f, "palette {palette_id} is not one of this state's")
            }

            Error::UnknownValuePool { value_pool_id } => {
                write!(f, "value pool {value_pool_id} is not one of this state's")
            }

            Error::UnknownHierarchyNode { node_id } => {
                write!(f, "hierarchy node {node_id} is not one of this state's")
            }

            Error::UnknownProperty { property_id } => {
                write!(f, "property {property_id} is not one of the palette's")
            }

            Error::UnknownMaterial { material_id } => {
                write!(f, "material {material_id} is not one of the palette's")
            }

            Error::UnknownValuePoolValue { value_id } => {
                write!(f, "value {value_id} is not one of the value pool's")
            }

            Error::UnknownLayer { layer_id } => {
                write!(f, "layer {layer_id} is not one of the object's")
            }

            Error::UnknownVoxel { voxel_id } => {
                write!(f, "voxel {voxel_id} is outside the object's grid")
            }

            Error::RemappedVoxelOutsideGrid {
                voxel_id,
                position,
                bounds,
            } => write!(
                f,
                "voxel {} would move to [{}, {}, {}], outside the {} x {} x {} grid",
                voxel_id, position.x, position.y, position.z, bounds.x, bounds.y, bounds.z
            ),

            Error::RemappedVoxelCollision {
                voxel_id,
                other_voxel_id,
            } => write!(
                f,
                "voxels {other_voxel_id} and {voxel_id} would move onto one cell"
            ),

            Error::ResampleSourceOutsideGrid { position, bounds } => write!(
                f,
                "a resampled cell draws from [{}, {}, {}], outside the {} x {} x {} grid",
                position.x, position.y, position.z, bounds.x, bounds.y, bounds.z
            ),

            Error::NonColorProperty {
                palette_id,
                property_id,
            } => write!(
                f,
                "palette {palette_id} property {property_id} draws from a value pool that holds no \
                 colors"
            ),

            Error::IndexPastCount { index, count } => {
                write!(f, "index {index} is at or past the listing count {count}")
            }

            Error::MaterialInUse {
                material_id,
                object_ids,
            } => write!(
                f,
                "material {} is still sampled by objects {}",
                material_id,
                object_ids.display_ids()
            ),

            Error::ValuePoolValueInUse {
                value_id,
                palette_ids,
            } => write!(
                f,
                "value {} is still drawn by palettes {}",
                value_id,
                palette_ids.display_ids()
            ),

            Error::PaletteInUse {
                palette_id,
                object_ids,
            } => write!(
                f,
                "palette {} is still referenced by objects {}",
                palette_id,
                object_ids.display_ids()
            ),

            Error::ValuePoolInUse {
                value_pool_id,
                palette_ids,
            } => write!(
                f,
                "value pool {} is still referenced by palettes {}",
                value_pool_id,
                palette_ids.display_ids()
            ),

            Error::ObjectInUse {
                object_id,
                node_ids,
            } => write!(
                f,
                "object {} is still placed by hierarchy nodes {}",
                object_id,
                node_ids.display_ids()
            ),

            Error::HierarchyNodeInUse {
                node_id,
                parent_ids,
                root,
            } => write!(
                f,
                "hierarchy node {} is still referenced: parents {}, root {}",
                node_id,
                parent_ids.display_ids(),
                root
            ),

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

            Error::PropertyWithoutDefault { materials } => {
                write!(
                    f,
                    "the palette has {materials} materials, which a property needs a default \
                     value to fill"
                )
            }

            Error::LayerWithoutDefault { live_voxels } => {
                write!(
                    f,
                    "the object has {live_voxels} live voxels, which a layer needs a default \
                     material to fill"
                )
            }

            Error::PropertyValuePoolRef {
                property_id,
                value_pool_id,
            } => write!(
                f,
                "the inserted palette's property {property_id} names value pool {value_pool_id}, \
                 which is not one of this state's"
            ),

            Error::MaterialValueRef {
                property_id,
                material_id,
            } => write!(
                f,
                "the inserted palette's material {material_id} draws a value for property \
                 {property_id} that is not one of its value pool's"
            ),

            Error::LayerPaletteRef {
                layer_id,
                palette_id,
            } => write!(
                f,
                "the inserted object's layer {layer_id} references palette {palette_id}, which is \
                 not one of this state's"
            ),

            Error::LayerSampleMaterial {
                layer_id,
                voxel_id,
                material_id,
            } => write!(
                f,
                "voxel {voxel_id} samples material {material_id} for layer {layer_id}, which is \
                 not one of the layer's palette's"
            ),

            Error::InsertedDuplicateChildNode { index, child_id } => write!(
                f,
                "the hierarchy node at listing index {index} lists child node {child_id} more than \
                 once"
            ),

            Error::InsertedDuplicateChildObject { index, object_id } => write!(
                f,
                "the hierarchy node at listing index {index} places object {object_id} more than \
                 once"
            ),

            Error::InsertedNonFiniteTransform { index } => write!(
                f,
                "the hierarchy node at listing index {index} has a non-finite transform position \
                 or scale component"
            ),

            Error::InsertedZeroScale { index } => write!(
                f,
                "the hierarchy node at listing index {index} has a zero transform scale component"
            ),

            Error::InsertedNonUnitRotation { index } => write!(
                f,
                "the hierarchy node at listing index {index} has a transform rotation that is not \
                 a unit quaternion"
            ),

            Error::InsertedCycle { index } => write!(
                f,
                "the hierarchy nodes contain a cycle reaching the node at listing index {index}"
            ),

            Error::ValuePoolValue {
                value_pool_id,
                value_id,
            } => write!(
                f,
                "value pool {value_pool_id} value {value_id} is outside its kind's value domain"
            ),

            Error::PropertyValuePool {
                palette_id,
                property_id,
                value_pool_id,
            } => write!(
                f,
                "palette {palette_id} property {property_id} references value pool \
                 {value_pool_id}, which does not exist"
            ),

            Error::MaterialValue {
                palette_id,
                property_id,
                material_id,
            } => write!(
                f,
                "palette {palette_id} material {material_id} has a value id for property \
                 {property_id} that is not one of the value pool's values"
            ),

            Error::PaletteRef {
                object_id,
                palette_id,
            } => write!(
                f,
                "object {object_id} references palette {palette_id}, which does not exist"
            ),

            Error::SampleMaterial {
                object_id,
                voxel_id,
                material_id,
            } => write!(
                f,
                "object {object_id} voxel {voxel_id} samples material {material_id}, out of range \
                 of its palette"
            ),

            Error::ChildNode { node_id, child_id } => write!(
                f,
                "hierarchy node {node_id} lists child node {child_id}, which does not exist"
            ),

            Error::ChildObject { node_id, object_id } => write!(
                f,
                "hierarchy node {node_id} places object {object_id}, which does not exist"
            ),

            Error::Root { root_id } => write!(
                f,
                "root references hierarchy node {root_id}, which does not exist"
            ),

            Error::Cycle { node_id } => write!(
                f,
                "hierarchy is not acyclic: a cycle reaches node {node_id}"
            ),

            Error::DuplicateChildNode { node_id, child_id } => write!(
                f,
                "hierarchy node {node_id} lists child node {child_id} more than once"
            ),

            Error::DuplicateChildObject { node_id, object_id } => write!(
                f,
                "hierarchy node {node_id} places object {object_id} more than once"
            ),

            Error::DuplicateRoot { root_id } => {
                write!(f, "root lists hierarchy node {root_id} more than once")
            }

            Error::NonFiniteTransform { node_id } => write!(
                f,
                "hierarchy node {node_id} has a non-finite transform position or scale component"
            ),

            Error::ZeroScale { node_id } => write!(
                f,
                "hierarchy node {node_id} has a zero transform scale component"
            ),

            Error::NonUnitRotation { node_id } => write!(
                f,
                "hierarchy node {node_id} transform rotation is not a unit quaternion"
            ),
        }
    }
}

impl StdError for Error {}
