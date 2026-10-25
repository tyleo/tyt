use crate::ForwardDependencies;
use sdfj_sdfcore::codec::dependencies::{
    DecodeFont, DecodeSdfjJson, Deflate, EncodeSdfjJson, FindInstalledFont, Inflate, ReadFontFile,
};

/// The SDF Json codec's dependencies a caller supplies.
pub trait SdfjDependencies {
    /// The SDF Json codec's dependencies.
    type Sdfj: DecodeFont
        + DecodeSdfjJson
        + EncodeSdfjJson
        + Deflate
        + FindInstalledFont
        + Inflate
        + ReadFontFile;

    /// The SDF Json codec's dependencies.
    fn sdfj(&self) -> &Self::Sdfj;
}

/// Forwards to the target's.
impl<T: ForwardDependencies<Target: SdfjDependencies>> SdfjDependencies for T {
    type Sdfj = <T::Target as SdfjDependencies>::Sdfj;

    fn sdfj(&self) -> &Self::Sdfj {
        self.target().sdfj()
    }
}
