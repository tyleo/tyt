use crate::{BRenderLight, BRenderMaterial, BRenderPlacement, BRenderView};
use branded_id::U32Id;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use voxcore::{BVoxObject, BVoxVoxel, Error as VoxError, VoxObject};

/// An error from voxrender.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// voxcore refused a read of the flattened document.
    Vox { error: VoxError },

    /// An object grid of this many cells would exceed
    /// [`MAX_GRID_CELLS`](VoxObject::MAX_GRID_CELLS).
    GridCellCap { cells: u64 },

    /// A flatten named an object that is not one of the document's.
    UnknownVoxObject { object_id: U32Id<BVoxObject> },

    /// A flatten read a shaded property whose value pool is not the kind the
    /// contract reads: floats for a scalar, float vectors for a color.
    MaterialPropertyKind { property: String },

    /// A flatten's voxel size is not finite and positive.
    VoxelSize { voxel_size: f64 },

    /// A render asked for an image with a zero side.
    ImageSide { width: u32, height: u32 },

    /// A mutation named a material that is not one of the scene's.
    UnknownMaterial { material_id: U32Id<BRenderMaterial> },

    /// A mutation named an object that is not one of the scene's.
    UnknownObject { object_id: U32Id<BVoxObject> },

    /// A retain named an object the scene already holds.
    DuplicateObject { object_id: U32Id<BVoxObject> },

    /// A mutation named a placement that is not one of the scene's.
    UnknownPlacement {
        placement_id: U32Id<BRenderPlacement>,
    },

    /// A mutation named a light that is not one of the scene's.
    UnknownLight { light_id: U32Id<BRenderLight> },

    /// A mutation named a view that is not one of the scene's.
    UnknownView { view_id: U32Id<BRenderView> },

    /// A mutation named a voxel outside its object's grid.
    UnknownVoxel { voxel_id: U32Id<BVoxVoxel> },

    /// A retained object's voxel references a material that is not one of
    /// the scene's.
    VoxelMaterialRef {
        voxel_id: U32Id<BVoxVoxel>,

        material_id: U32Id<BRenderMaterial>,
    },

    /// A release found the material still sampled by these objects' voxels.
    MaterialInUse {
        material_id: U32Id<BRenderMaterial>,

        object_ids: Vec<U32Id<BVoxObject>>,
    },

    /// A release found the object still placed by these placements.
    ObjectInUse {
        object_id: U32Id<BVoxObject>,

        placement_ids: Vec<U32Id<BRenderPlacement>>,
    },

    /// A material's value for `property` is outside the range its name in
    /// voxcore's `material` module fixes, or is not finite.
    MaterialOutOfRange { property: String },

    /// A placement's position or scale is not finite.
    NonFinitePlacement,

    /// A placement's scale has a zero component.
    ZeroPlacementScale,

    /// A placement's rotation is not unit length.
    NonUnitPlacementRotation,

    /// A light's position, color, strength, range, or cone angle is not
    /// finite.
    NonFiniteLight,

    /// A directional or spot light's rotation is not unit length.
    NonUnitLightRotation,

    /// A light's color has a negative component or its strength is
    /// negative.
    NegativeLight,

    /// A point or spot light's range is not positive.
    NonPositiveLightRange { range: f64 },

    /// A spot light's cone angles are not ordered from zero, through the
    /// inner angle, to an outer angle of at most a quarter turn.
    SpotCone { inner_cone: f64, outer_cone: f64 },

    /// A view's position is not finite.
    NonFiniteView,

    /// A view's rotation is not unit length.
    NonUnitViewRotation,

    /// A perspective view's field of view is not within `(0, pi)`.
    FieldOfViewOutOfRange { fov: f64 },

    /// An orthographic view's scale is not finite and positive.
    ViewScale { scale: f64 },

    /// A bloom's strength is not finite and zero or more.
    BloomStrength { strength: f64 },

    /// A bloom's radius is not finite and positive.
    BloomRadius { radius: f64 },

    /// A bloom's threshold is not finite and zero or more.
    BloomThreshold { threshold: f64 },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // Ids print as their bare `u32`: a branded id's `Display` carries the
        // brand name, which the surrounding wording already gives.
        match self {
            Error::Vox { error } => write!(f, "voxcore: {error}"),

            Error::GridCellCap { cells } => write!(
                f,
                "a {cells}-cell grid exceeds the {}-cell dense cap",
                VoxObject::MAX_GRID_CELLS
            ),

            Error::UnknownVoxObject { object_id } => write!(
                f,
                "object {} is not one of the document's",
                object_id.to_u32()
            ),

            Error::MaterialPropertyKind { property } => write!(
                f,
                "property {property} is not the kind the render contract reads"
            ),

            Error::VoxelSize { voxel_size } => {
                write!(f, "voxel size {voxel_size} is not finite and positive")
            }

            Error::ImageSide { width, height } => {
                write!(f, "a {width} by {height} image has a zero side")
            }

            Error::UnknownMaterial { material_id } => write!(
                f,
                "material {} is not one of the scene's",
                material_id.to_u32()
            ),

            Error::UnknownObject { object_id } => {
                write!(f, "object {} is not one of the scene's", object_id.to_u32())
            }

            Error::DuplicateObject { object_id } => write!(
                f,
                "object {} is already one of the scene's",
                object_id.to_u32()
            ),

            Error::UnknownPlacement { placement_id } => write!(
                f,
                "placement {} is not one of the scene's",
                placement_id.to_u32()
            ),

            Error::UnknownLight { light_id } => {
                write!(f, "light {} is not one of the scene's", light_id.to_u32())
            }

            Error::UnknownView { view_id } => {
                write!(f, "view {} is not one of the scene's", view_id.to_u32())
            }

            Error::UnknownVoxel { voxel_id } => {
                write!(f, "voxel {} is outside the grid", voxel_id.to_u32())
            }

            Error::VoxelMaterialRef {
                voxel_id,
                material_id,
            } => write!(
                f,
                "voxel {} samples material {}, which is not one of the scene's",
                voxel_id.to_u32(),
                material_id.to_u32()
            ),

            Error::MaterialInUse {
                material_id,
                object_ids,
            } => write!(
                f,
                "material {} is still sampled by {} object(s)",
                material_id.to_u32(),
                object_ids.len()
            ),

            Error::ObjectInUse {
                object_id,
                placement_ids,
            } => write!(
                f,
                "object {} is still placed by {} placement(s)",
                object_id.to_u32(),
                placement_ids.len()
            ),

            Error::MaterialOutOfRange { property } => {
                write!(f, "material property {property} is outside its range")
            }

            Error::NonFinitePlacement => write!(f, "a placement's transform is not finite"),

            Error::ZeroPlacementScale => write!(f, "a placement's scale has a zero component"),

            Error::NonUnitPlacementRotation => {
                write!(f, "a placement's rotation is not unit length")
            }

            Error::NonFiniteLight => write!(f, "a light's values are not finite"),

            Error::NonUnitLightRotation => write!(f, "a light's rotation is not unit length"),

            Error::NegativeLight => write!(f, "a light's color or strength is negative"),

            Error::NonPositiveLightRange { range } => {
                write!(f, "light range {range} is not positive")
            }

            Error::SpotCone {
                inner_cone,
                outer_cone,
            } => write!(
                f,
                "spot cone angles {inner_cone} and {outer_cone} radians are not ordered from 0 \
                 through the inner to an outer of at most pi/2"
            ),

            Error::NonFiniteView => write!(f, "a view's position is not finite"),

            Error::NonUnitViewRotation => write!(f, "a view's rotation is not unit length"),

            Error::FieldOfViewOutOfRange { fov } => {
                write!(f, "field of view {fov} is not within (0, pi) radians")
            }

            Error::ViewScale { scale } => {
                write!(f, "view scale {scale} is not finite and positive")
            }

            Error::BloomStrength { strength } => {
                write!(
                    f,
                    "bloom strength {strength} is not finite and zero or more"
                )
            }

            Error::BloomRadius { radius } => {
                write!(f, "bloom radius {radius} is not finite and positive")
            }

            Error::BloomThreshold { threshold } => {
                write!(
                    f,
                    "bloom threshold {threshold} is not finite and zero or more"
                )
            }
        }
    }
}

impl StdError for Error {}

impl From<VoxError> for Error {
    fn from(error: VoxError) -> Self {
        Error::Vox { error }
    }
}
