use branded_id::U32Id;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use voxrender::{BRenderMaterial, Error as RenderError};
use voxrender_wgpu::Error as WgpuError;
use wgpu::{BufferAsyncError, Features, MapRangeError, PollError};

/// Why a benchmark failed to run.
#[derive(Debug)]
pub enum Error {
    /// The reference could not set up a scene's rays.
    Render(RenderError),

    /// The GPU tier's trace failed.
    Trace(WgpuError),

    /// No bench scene's name starts with the text.
    NoScene(String),

    /// The scene has no view to draw.
    NoView,

    /// The device lacks timestamp queries, which a timed benchmark needs.
    NoTimestampQuery,

    /// No kernel's label holds the text a timed benchmark was asked for.
    NoKernel(String),

    /// The device lacks ray queries, which the ray-query trace needs.
    NoRayQuery,

    /// The device lacks features a raster arm needs.
    RasterFeatures(Features),

    /// A raster's 32-bit hit has no room for a scene's placements beside its
    /// voxel ids.
    RasterHit { placements: usize, voxel_bits: u32 },

    /// Waiting on the GPU failed.
    Poll(PollError),

    /// A readback would not map.
    Map(BufferAsyncError),

    /// A mapped readback range was not available.
    MapRange(MapRangeError),

    /// A pass's end timestamp came before its start.
    TimestampOrder,

    /// A material id is past what a tree's slot holds.
    TreeMaterial { material_id: U32Id<BRenderMaterial> },

    /// A tree's word offset is past what its slot holds.
    TreeSize { offset: usize },

    /// A tile holds more materials than its cells' entries index.
    TileMaterials { materials: usize },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Render(error) => write!(f, "the reference failed: {error}"),
            Error::Trace(error) => write!(f, "the GPU trace failed: {error}"),
            Error::NoScene(name) => write!(f, "no bench scene's name starts with {name:?}"),
            Error::NoView => write!(f, "the scene has no view"),
            Error::NoTimestampQuery => write!(f, "the device lacks timestamp queries"),
            Error::NoKernel(text) => write!(f, "no kernel's label holds {text:?}"),
            Error::NoRayQuery => write!(f, "the device lacks ray queries"),
            Error::RasterFeatures(features) => {
                write!(f, "the device lacks a raster arm's features {features:?}")
            }
            Error::RasterHit {
                placements,
                voxel_bits,
            } => write!(
                f,
                "{placements} placements do not fit a raster's hit beside {voxel_bits} voxel bits"
            ),
            Error::Poll(error) => write!(f, "waiting on the GPU failed: {error}"),
            Error::Map(error) => write!(f, "a readback would not map: {error}"),
            Error::MapRange(error) => write!(f, "a readback range was unavailable: {error}"),
            Error::TimestampOrder => write!(f, "a pass ended before it began"),
            Error::TreeMaterial { material_id } => {
                write!(f, "material {material_id} is past what a tree's slot holds")
            }
            Error::TreeSize { offset } => {
                write!(f, "word offset {offset} is past what a tree's slot holds")
            }
            Error::TileMaterials { materials } => {
                write!(f, "a tile holds {materials} materials, over 256")
            }
        }
    }
}

impl StdError for Error {}
