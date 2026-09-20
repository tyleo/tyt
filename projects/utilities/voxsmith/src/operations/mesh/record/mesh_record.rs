use crate::operations::mesh::{
    ComputedBinding, ExtraWrite, FileWrite, MaterialRecord, Method, PrimitiveRecord, TextureShape,
};
use branded_id::IdVec;
use meshdoc::{BMeshMaterial, BMeshPrimitive};

/// A whole meshing run, with the flags and profiles lowered in.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshRecord {
    /// The meshing strategy.
    pub method: Method,

    /// The atlas canvas.
    pub texture_shape: TextureShape,

    /// One voxel's edge length in meters.
    pub voxel_size: f64,

    /// The computed bindings.
    pub computed_bindings: Vec<ComputedBinding>,

    /// The joined value program.
    pub program: String,

    /// The materials by index; the material count sets the length.
    pub materials: IdVec<BMeshMaterial, MaterialRecord>,

    /// The primitives by index; the implicit primitive lowers as an entry
    /// whose `true` select takes every face, so the table holds at least one.
    pub primitives: IdVec<BMeshPrimitive, PrimitiveRecord>,

    /// The loose files written beside the mesh.
    pub files: Vec<FileWrite>,

    /// The object's named properties.
    pub mesh_extras: Vec<ExtraWrite>,
}
