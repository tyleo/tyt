use crate::{
    BMeshFile, BMeshHierarchyNode, BMeshImage, BMeshMaterial, BMeshObject, BMeshPrimitive,
    BMeshTexture, BMeshTriangle, BMeshUvStream, BMeshVertex, MeshImageMediaType,
};
use branded_id::{SliceExt, U32Id};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error from meshdoc.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// The ext could not follow a mutation. A `will` hook's refusal leaves the
    /// state unchanged.
    Ext { reason: String },

    /// A listing of this many entries has more than a `u32` id addresses.
    ListingCap { entries: u64 },

    /// A mutation referenced a file that is not one of the state's.
    UnknownFile { file_id: U32Id<BMeshFile> },

    /// A mutation referenced an image that is not one of the state's.
    UnknownImage { image_id: U32Id<BMeshImage> },

    /// A mutation referenced a texture that is not one of the state's.
    UnknownTexture { texture_id: U32Id<BMeshTexture> },

    /// A mutation referenced a material that is not one of the state's.
    UnknownMaterial { material_id: U32Id<BMeshMaterial> },

    /// A mutation referenced an object that is not one of the state's.
    UnknownObject { object_id: U32Id<BMeshObject> },

    /// A mutation referenced a primitive that is not one of the object's.
    UnknownPrimitive { primitive_id: U32Id<BMeshPrimitive> },

    /// A mutation referenced a hierarchy node that is not one of the state's.
    UnknownHierarchyNode { node_id: U32Id<BMeshHierarchyNode> },

    /// A move targeted a listing position at or past the listing's count.
    IndexPastCount { index: usize, count: usize },

    /// A release targeted a file still referenced: `image_ids` lists the images
    /// reading it, and `material_ids` and `object_ids` the materials and
    /// objects with a property pointing at it, each in listing order.
    FileInUse {
        file_id: U32Id<BMeshFile>,

        image_ids: Vec<U32Id<BMeshImage>>,

        material_ids: Vec<U32Id<BMeshMaterial>>,

        object_ids: Vec<U32Id<BMeshObject>>,
    },

    /// A release targeted an image a texture still samples. `texture_ids`
    /// lists the sampling textures in listing order.
    ImageInUse {
        image_id: U32Id<BMeshImage>,

        texture_ids: Vec<U32Id<BMeshTexture>>,
    },

    /// A release targeted a texture still referenced: `material_ids` lists the
    /// materials drawing it and `object_ids` the objects with a property
    /// referencing it, each in listing order.
    TextureInUse {
        texture_id: U32Id<BMeshTexture>,

        material_ids: Vec<U32Id<BMeshMaterial>>,

        object_ids: Vec<U32Id<BMeshObject>>,
    },

    /// A release targeted a material a primitive still draws with.
    /// `object_ids` lists the objects of those primitives in listing order.
    MaterialInUse {
        material_id: U32Id<BMeshMaterial>,

        object_ids: Vec<U32Id<BMeshObject>>,
    },

    /// A release targeted an object a hierarchy node still places. `node_ids`
    /// lists the placing nodes in listing order.
    ObjectInUse {
        object_id: U32Id<BMeshObject>,

        node_ids: Vec<U32Id<BMeshHierarchyNode>>,
    },

    /// A release targeted a hierarchy node still referenced: `parent_ids` lists
    /// its parents, in listing order, and `root` reports whether the roots
    /// list it.
    HierarchyNodeInUse {
        node_id: U32Id<BMeshHierarchyNode>,

        parent_ids: Vec<U32Id<BMeshHierarchyNode>>,

        root: bool,
    },

    /// A vertex stream was given an entry count different from the vertex
    /// count.
    StreamArity { entries: usize, vertices: usize },

    /// A vertex attribute has a zero width or a component count other than
    /// its width per vertex.
    AttributeArity {
        name: String,

        components: usize,

        expected: usize,
    },

    /// A vertex attribute was given a name the primitive already uses.
    DuplicateVertexAttributeName { name: String },

    /// A triangle corner references a vertex past the primitive's vertices.
    CornerVertex {
        triangle_id: U32Id<BMeshTriangle>,

        vertex_id: U32Id<BMeshVertex>,
    },

    /// A vertex stream holds a non-finite value at this vertex.
    NonFiniteVertex { vertex_id: U32Id<BMeshVertex> },

    /// An inserted or replaced file has an empty name.
    EmptyFileName,

    /// An inserted or replaced file has a name file `file_id` already uses.
    FileNameTaken {
        name: String,

        file_id: U32Id<BMeshFile>,
    },

    /// An inserted or replaced image reads a file that is not one of the
    /// state's.
    ImageFileRef { file_id: U32Id<BMeshFile> },

    /// An inserted or replaced image's bytes, or the bytes of a file an
    /// image reads, do not start with the image's media type's signature.
    MalformedImage { media_type: MeshImageMediaType },

    /// An inserted or replaced material has this factor outside the range
    /// its name fixes.
    MaterialFactor { name: String, value: f64 },

    /// A material or object was given two properties of the same name.
    DuplicatePropertyName { name: String },

    /// A material or object property holds a non-finite float.
    NonFiniteProperty { name: String },

    /// A material or object property references a texture that is not one
    /// of the state's.
    PropertyTextureRef {
        name: String,

        texture_id: U32Id<BMeshTexture>,
    },

    /// A material or object property points at a file that is not one of
    /// the state's.
    PropertyFileRef {
        name: String,

        file_id: U32Id<BMeshFile>,
    },

    /// An inserted or replaced texture samples an image that is not one of
    /// the state's.
    TextureImageRef { image_id: U32Id<BMeshImage> },

    /// An inserted or replaced material draws a texture that is not one of
    /// the state's.
    MaterialTextureRef { texture_id: U32Id<BMeshTexture> },

    /// An inserted primitive draws with a material that is not one of the
    /// state's.
    PrimitiveMaterialRef {
        primitive_id: U32Id<BMeshPrimitive>,

        material_id: U32Id<BMeshMaterial>,
    },

    /// A primitive draws with a material whose texture samples a UV stream
    /// the primitive does not carry.
    PrimitiveUvStreamRef {
        primitive_id: U32Id<BMeshPrimitive>,

        material_id: U32Id<BMeshMaterial>,

        uv_stream_id: U32Id<BMeshUvStream>,
    },

    /// An inserted or replaced hierarchy node, at this listing index in its
    /// batch, lists the same child node more than once.
    InsertedDuplicateChildNode {
        index: usize,

        child_id: U32Id<BMeshHierarchyNode>,
    },

    /// An inserted or replaced hierarchy node, at this listing index in its
    /// batch, places the same object more than once.
    InsertedDuplicateChildObject {
        index: usize,

        object_id: U32Id<BMeshObject>,
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

    /// A file has an empty name.
    FileName { file_id: U32Id<BMeshFile> },

    /// Two files share a name.
    DuplicateFileName {
        file_id: U32Id<BMeshFile>,

        other_file_id: U32Id<BMeshFile>,
    },

    /// An image reads a file that does not exist.
    ImageFile {
        image_id: U32Id<BMeshImage>,

        file_id: U32Id<BMeshFile>,
    },

    /// An image's bytes do not start with its media type's signature.
    ImageBytes { image_id: U32Id<BMeshImage> },

    /// A texture samples an image that does not exist.
    TextureImage {
        texture_id: U32Id<BMeshTexture>,

        image_id: U32Id<BMeshImage>,
    },

    /// A material has this factor outside the range its name fixes.
    MaterialFactorRange {
        material_id: U32Id<BMeshMaterial>,

        name: String,
    },

    /// A material draws a texture that does not exist.
    MaterialTexture {
        material_id: U32Id<BMeshMaterial>,

        texture_id: U32Id<BMeshTexture>,
    },

    /// A primitive draws with a material that does not exist.
    PrimitiveMaterial {
        object_id: U32Id<BMeshObject>,

        primitive_id: U32Id<BMeshPrimitive>,

        material_id: U32Id<BMeshMaterial>,
    },

    /// A primitive draws with a material whose texture samples a UV stream
    /// the primitive does not carry.
    PrimitiveUvStream {
        object_id: U32Id<BMeshObject>,

        primitive_id: U32Id<BMeshPrimitive>,

        material_id: U32Id<BMeshMaterial>,

        uv_stream_id: U32Id<BMeshUvStream>,
    },

    /// A node lists a child node that does not exist.
    ChildNode {
        node_id: U32Id<BMeshHierarchyNode>,

        child_id: U32Id<BMeshHierarchyNode>,
    },

    /// A node places an object that does not exist.
    ChildObject {
        node_id: U32Id<BMeshHierarchyNode>,

        object_id: U32Id<BMeshObject>,
    },

    /// A root references a node that does not exist.
    Root { root_id: U32Id<BMeshHierarchyNode> },

    /// The hierarchy contains a cycle reaching this node.
    Cycle { node_id: U32Id<BMeshHierarchyNode> },

    /// A node lists the same child node more than once.
    DuplicateChildNode {
        node_id: U32Id<BMeshHierarchyNode>,

        child_id: U32Id<BMeshHierarchyNode>,
    },

    /// A node places the same object more than once.
    DuplicateChildObject {
        node_id: U32Id<BMeshHierarchyNode>,

        object_id: U32Id<BMeshObject>,
    },

    /// A root lists the same node more than once.
    DuplicateRoot { root_id: U32Id<BMeshHierarchyNode> },

    /// A node's transform has a non-finite position or scale component.
    NonFiniteTransform { node_id: U32Id<BMeshHierarchyNode> },

    /// A node's transform has a zero scale component.
    ZeroScale { node_id: U32Id<BMeshHierarchyNode> },

    /// A node's transform rotation is not a unit quaternion.
    NonUnitRotation { node_id: U32Id<BMeshHierarchyNode> },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // Ids print as their bare `u32`: a branded id's `Display` carries the
        // brand name, which the surrounding wording already gives.
        match self {
            Error::Ext { reason } => write!(f, "ext: {reason}"),

            Error::ListingCap { entries } => write!(
                f,
                "a listing of {entries} entries has more than a u32 id addresses"
            ),

            Error::UnknownFile { file_id } => {
                write!(f, "file {file_id} is not one of this state's")
            }

            Error::UnknownImage { image_id } => {
                write!(f, "image {image_id} is not one of this state's")
            }

            Error::UnknownTexture { texture_id } => {
                write!(f, "texture {texture_id} is not one of this state's")
            }

            Error::UnknownMaterial { material_id } => {
                write!(f, "material {material_id} is not one of this state's")
            }

            Error::UnknownObject { object_id } => {
                write!(f, "object {object_id} is not one of this state's")
            }

            Error::UnknownPrimitive { primitive_id } => {
                write!(f, "primitive {primitive_id} is not one of the object's")
            }

            Error::UnknownHierarchyNode { node_id } => {
                write!(f, "hierarchy node {node_id} is not one of this state's")
            }

            Error::IndexPastCount { index, count } => {
                write!(f, "index {index} is at or past the listing count {count}")
            }

            Error::FileInUse {
                file_id,
                image_ids,
                material_ids,
                object_ids,
            } => write!(
                f,
                "file {} is still referenced: images {}, materials {}, objects {}",
                file_id,
                image_ids.display_ids(),
                material_ids.display_ids(),
                object_ids.display_ids()
            ),

            Error::ImageInUse {
                image_id,
                texture_ids,
            } => write!(
                f,
                "image {} is still sampled by textures {}",
                image_id,
                texture_ids.display_ids()
            ),

            Error::TextureInUse {
                texture_id,
                material_ids,
                object_ids,
            } => write!(
                f,
                "texture {} is still referenced: materials {}, objects {}",
                texture_id,
                material_ids.display_ids(),
                object_ids.display_ids()
            ),

            Error::MaterialInUse {
                material_id,
                object_ids,
            } => write!(
                f,
                "material {} is still drawn by primitives of objects {}",
                material_id,
                object_ids.display_ids()
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

            Error::StreamArity { entries, vertices } => {
                write!(f, "{entries} entries were given for {vertices} vertices")
            }

            Error::AttributeArity {
                name,
                components,
                expected,
            } => write!(
                f,
                "vertex attribute \"{name}\" holds {components} components, not the {expected} its \
                 width gives over the vertices"
            ),

            Error::DuplicateVertexAttributeName { name } => {
                write!(f, "a vertex attribute named \"{name}\" already exists")
            }

            Error::CornerVertex {
                triangle_id,
                vertex_id,
            } => write!(
                f,
                "triangle {triangle_id} references vertex {vertex_id}, past the primitive's \
                 vertices"
            ),

            Error::NonFiniteVertex { vertex_id } => {
                write!(f, "vertex {vertex_id} holds a non-finite value")
            }

            Error::EmptyFileName => write!(f, "the file has an empty name"),

            Error::FileNameTaken { name, file_id } => write!(
                f,
                "the file name \"{name}\" is already used by file {file_id}"
            ),

            Error::ImageFileRef { file_id } => write!(
                f,
                "the image reads file {file_id}, which is not one of this state's"
            ),

            Error::MalformedImage { media_type } => write!(
                f,
                "the image bytes do not start with the {media_type} signature"
            ),

            Error::MaterialFactor { name, value } => write!(
                f,
                "material factor \"{name}\" is {value}, outside its range"
            ),

            Error::DuplicatePropertyName { name } => {
                write!(f, "a property named \"{name}\" already exists")
            }

            Error::NonFiniteProperty { name } => {
                write!(f, "property \"{name}\" holds a non-finite value")
            }

            Error::PropertyTextureRef { name, texture_id } => write!(
                f,
                "property \"{name}\" references texture {texture_id}, which is not one of this \
                 state's"
            ),

            Error::PropertyFileRef { name, file_id } => write!(
                f,
                "property \"{name}\" points at file {file_id}, which is not one of this state's"
            ),

            Error::TextureImageRef { image_id } => write!(
                f,
                "the texture samples image {image_id}, which is not one of this state's"
            ),

            Error::MaterialTextureRef { texture_id } => write!(
                f,
                "the material draws texture {texture_id}, which is not one of this state's"
            ),

            Error::PrimitiveMaterialRef {
                primitive_id,
                material_id,
            } => write!(
                f,
                "primitive {primitive_id} draws with material {material_id}, which is not one of \
                 this state's"
            ),

            Error::PrimitiveUvStreamRef {
                primitive_id,
                material_id,
                uv_stream_id,
            } => write!(
                f,
                "primitive {primitive_id} draws with material {material_id}, whose texture samples \
                 UV stream {uv_stream_id}, which the primitive does not carry"
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

            Error::FileName { file_id } => {
                write!(f, "file {file_id} has an empty name")
            }

            Error::DuplicateFileName {
                file_id,
                other_file_id,
            } => write!(
                f,
                "file {file_id} shares its name with file {other_file_id}"
            ),

            Error::ImageFile { image_id, file_id } => write!(
                f,
                "image {image_id} reads file {file_id}, which does not exist"
            ),

            Error::ImageBytes { image_id } => write!(
                f,
                "image {image_id} does not start with its media type's signature"
            ),

            Error::TextureImage {
                texture_id,
                image_id,
            } => write!(
                f,
                "texture {texture_id} samples image {image_id}, which does not exist"
            ),

            Error::MaterialFactorRange { material_id, name } => write!(
                f,
                "material {material_id} factor \"{name}\" is outside its range"
            ),

            Error::MaterialTexture {
                material_id,
                texture_id,
            } => write!(
                f,
                "material {material_id} draws texture {texture_id}, which does not exist"
            ),

            Error::PrimitiveMaterial {
                object_id,
                primitive_id,
                material_id,
            } => write!(
                f,
                "object {object_id} primitive {primitive_id} draws with material {material_id}, \
                 which does not exist"
            ),

            Error::PrimitiveUvStream {
                object_id,
                primitive_id,
                material_id,
                uv_stream_id,
            } => write!(
                f,
                "object {object_id} primitive {primitive_id} draws with material {material_id}, \
                 whose texture samples UV stream {uv_stream_id}, which the primitive does not \
                 carry"
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
