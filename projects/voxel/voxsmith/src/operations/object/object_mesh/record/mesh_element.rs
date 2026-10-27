use branded_id::U32Id;
use meshdoc::{BMeshMaterial, BMeshPrimitive};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// The element of a [`MeshRecord`](crate::operations::object::MeshRecord) an
/// error rose from.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum MeshElement {
    /// A computed binding, by its bound name.
    ComputedBinding {
        /// The bound name.
        name: String,
    },

    /// A file write, by its file.
    File {
        /// The file's relative name.
        file: String,
    },

    /// A material's extras entry.
    MaterialExtra {
        /// The material.
        material_id: U32Id<BMeshMaterial>,

        /// The entry's name.
        name: String,
    },

    /// A material's name.
    MaterialName {
        /// The material.
        material_id: U32Id<BMeshMaterial>,
    },

    /// A material's declared stream list.
    MaterialUvStreams {
        /// The material.
        material_id: U32Id<BMeshMaterial>,
    },

    /// The material table.
    Materials,

    /// The object's extras entry.
    MeshExtra {
        /// The entry's name.
        name: String,
    },

    /// The meshing strategy.
    Method,

    /// A primitive's vertex attribute write.
    PrimitiveAttribute {
        /// The primitive.
        primitive_id: U32Id<BMeshPrimitive>,

        /// The attribute's name.
        name: String,
    },

    /// A primitive's material.
    PrimitiveMaterial {
        /// The primitive.
        primitive_id: U32Id<BMeshPrimitive>,
    },

    /// A primitive's name.
    PrimitiveName {
        /// The primitive.
        primitive_id: U32Id<BMeshPrimitive>,
    },

    /// Whether a primitive writes its normals.
    PrimitiveNormal {
        /// The primitive.
        primitive_id: U32Id<BMeshPrimitive>,
    },

    /// A primitive's select.
    PrimitiveSelect {
        /// The primitive.
        primitive_id: U32Id<BMeshPrimitive>,
    },

    /// A primitive's declared stream list.
    PrimitiveUvStreams {
        /// The primitive.
        primitive_id: U32Id<BMeshPrimitive>,
    },

    /// The primitive table.
    Primitives,

    /// The value program.
    Program,

    /// A material's slot write.
    Slot {
        /// The material.
        material_id: U32Id<BMeshMaterial>,

        /// The destination property.
        property: String,
    },

    /// The atlas canvas.
    TextureShape,

    /// The uniform scale on the scene.
    SceneScale,
}

impl Display for MeshElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            MeshElement::ComputedBinding { name } => write!(f, "the computed binding `{name}`"),

            MeshElement::File { file } => write!(f, "the file `{file}`"),

            MeshElement::MaterialExtra { material_id, name } => {
                write!(f, "material {material_id}'s extra `{name}`")
            }

            MeshElement::MaterialName { material_id } => {
                write!(f, "material {material_id}'s name")
            }

            MeshElement::MaterialUvStreams { material_id } => {
                write!(f, "material {material_id}'s UV streams")
            }

            MeshElement::Materials => f.write_str("the material table"),

            MeshElement::MeshExtra { name } => write!(f, "the mesh extra `{name}`"),

            MeshElement::Method => f.write_str("the method"),

            MeshElement::PrimitiveAttribute { primitive_id, name } => {
                write!(f, "primitive {primitive_id}'s attribute `{name}`")
            }

            MeshElement::PrimitiveMaterial { primitive_id } => {
                write!(f, "primitive {primitive_id}'s material")
            }

            MeshElement::PrimitiveName { primitive_id } => {
                write!(f, "primitive {primitive_id}'s name")
            }

            MeshElement::PrimitiveNormal { primitive_id } => {
                write!(f, "primitive {primitive_id}'s normal")
            }

            MeshElement::PrimitiveSelect { primitive_id } => {
                write!(f, "primitive {primitive_id}'s select")
            }

            MeshElement::PrimitiveUvStreams { primitive_id } => {
                write!(f, "primitive {primitive_id}'s UV streams")
            }

            MeshElement::Primitives => f.write_str("the primitive table"),

            MeshElement::Program => f.write_str("the program"),

            MeshElement::Slot {
                material_id,
                property,
            } => write!(f, "material {material_id}'s slot `{property}`"),

            MeshElement::TextureShape => f.write_str("the texture shape"),

            MeshElement::SceneScale => f.write_str("the scene scale"),
        }
    }
}
