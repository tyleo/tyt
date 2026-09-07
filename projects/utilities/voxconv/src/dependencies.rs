#[cfg(feature = "goxl")]
use goxl_voxcore::codec::dependencies::{DecodePng as GoxlDecodePng, EncodePng as GoxlEncodePng};
#[cfg(feature = "qbcl")]
use qbcl_voxcore::codec::dependencies::{CompressZlib, DecompressZlib};
#[cfg(feature = "vmax")]
use vmax_voxcore::codec::dependencies::{
    CompressLzfse, DecodePng as VMaxDecodePng, DecodeVMaxPlist, DecodeVMaxSceneJson,
    DecompressLzfse, EncodePng as VMaxEncodePng, EncodeVMaxPlist, EncodeVMaxSceneJson,
};
#[cfg(feature = "voxj")]
use voxj::dependencies::{CostVoxjObject, DecodeBase64, EncodeBase64};
#[cfg(feature = "voxj")]
use voxj_voxcore::codec::dependencies::{DecodeVoxjJson, Deflate, EncodeVoxjJson, Inflate};

/// The codec dependencies the read and write functions take: one value per
/// enabled format, bound on that format's codec traits. The formats' codec
/// crates share trait names such as `DecodePng` with differing signatures,
/// so each format gets its own associated type instead of one bound over
/// all of them. MagicaVoxel's codec needs none.
pub trait Dependencies {
    /// The Goxel codec's dependencies.
    #[cfg(feature = "goxl")]
    type Goxl: GoxlDecodePng + GoxlEncodePng;

    /// The Goxel codec's dependencies.
    #[cfg(feature = "goxl")]
    fn goxl(&self) -> &Self::Goxl;

    /// The Qubicle codec's dependencies.
    #[cfg(feature = "qbcl")]
    type Qbcl: CompressZlib + DecompressZlib;

    /// The Qubicle codec's dependencies.
    #[cfg(feature = "qbcl")]
    fn qbcl(&self) -> &Self::Qbcl;

    /// The Voxel Max codec's dependencies.
    #[cfg(feature = "vmax")]
    type VMax: CompressLzfse
        + DecompressLzfse
        + DecodeVMaxPlist
        + EncodeVMaxPlist
        + VMaxDecodePng
        + VMaxEncodePng
        + DecodeVMaxSceneJson
        + EncodeVMaxSceneJson;

    /// The Voxel Max codec's dependencies.
    #[cfg(feature = "vmax")]
    fn vmax(&self) -> &Self::VMax;

    /// The Voxel Json codec's dependencies, covering voxj's too.
    #[cfg(feature = "voxj")]
    type Voxj: DecodeBase64
        + EncodeBase64
        + CostVoxjObject
        + DecodeVoxjJson
        + EncodeVoxjJson
        + Inflate
        + Deflate;

    /// The Voxel Json codec's dependencies.
    #[cfg(feature = "voxj")]
    fn voxj(&self) -> &Self::Voxj;
}
