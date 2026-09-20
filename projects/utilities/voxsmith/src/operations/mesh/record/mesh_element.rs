use branded_id::U32Id;
use meshdoc::{BMeshMaterial, BMeshPrimitive};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// The element of a [`MeshRecord`](crate::operations::mesh::MeshRecord) an
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

    /// One voxel's edge length.
    VoxelSize,
}

impl Display for MeshElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            MeshElement::ComputedBinding { name } => write!(f, "the computed binding `{name}`"),

            MeshElement::File { file } => write!(f, "the file `{file}`"),

            MeshElement::MaterialExtra { material_id, name } => {
                write!(f, "material {}'s extra `{name}`", material_id.to_u32())
            }

            MeshElement::MaterialName { material_id } => {
                write!(f, "material {}'s name", material_id.to_u32())
            }

            MeshElement::MaterialUvStreams { material_id } => {
                write!(f, "material {}'s UV streams", material_id.to_u32())
            }

            MeshElement::Materials => f.write_str("the material table"),

            MeshElement::MeshExtra { name } => write!(f, "the mesh extra `{name}`"),

            MeshElement::Method => f.write_str("the method"),

            MeshElement::PrimitiveAttribute { primitive_id, name } => {
                write!(
                    f,
                    "primitive {}'s attribute `{name}`",
                    primitive_id.to_u32()
                )
            }

            MeshElement::PrimitiveMaterial { primitive_id } => {
                write!(f, "primitive {}'s material", primitive_id.to_u32())
            }

            MeshElement::PrimitiveName { primitive_id } => {
                write!(f, "primitive {}'s name", primitive_id.to_u32())
            }

            MeshElement::PrimitiveNormal { primitive_id } => {
                write!(f, "primitive {}'s normal", primitive_id.to_u32())
            }

            MeshElement::PrimitiveSelect { primitive_id } => {
                write!(f, "primitive {}'s select", primitive_id.to_u32())
            }

            MeshElement::PrimitiveUvStreams { primitive_id } => {
                write!(f, "primitive {}'s UV streams", primitive_id.to_u32())
            }

            MeshElement::Primitives => f.write_str("the primitive table"),

            MeshElement::Program => f.write_str("the program"),

            MeshElement::Slot {
                material_id,
                property,
            } => write!(f, "material {}'s slot `{property}`", material_id.to_u32()),

            MeshElement::TextureShape => f.write_str("the texture shape"),

            MeshElement::VoxelSize => f.write_str("the voxel size"),
        }
    }
}
