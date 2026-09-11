#[cfg(feature = "goxl")]
use crate::goxl::Goxl;
#[cfg(feature = "mvox")]
use crate::mvox::MVox;
#[cfg(feature = "qbcl")]
use crate::qbcl::{Qb, Qbcl, Qbt};
#[cfg(feature = "vmax")]
use crate::vmax::VMax;
#[cfg(feature = "voxj")]
use crate::voxj::Voxj;
use crate::{InstalledFormat, ReadFormatVisitor};

/// A voxel document format a document is read as, one variant per enabled
/// format feature. Each variant stands for a [`Format`](crate::Format)
/// marker, and [`with`](ReadFormat::with) hands that marker to a visitor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReadFormat {
    /// Goxel, the `.gox` file.
    #[cfg(feature = "goxl")]
    Goxl,

    /// MagicaVoxel, the `.vox` file.
    #[cfg(feature = "mvox")]
    MVox,

    /// Qubicle Binary, the `.qb` file.
    #[cfg(feature = "qbcl")]
    Qb,

    /// Qubicle Binary Tree, the `.qbt` file.
    #[cfg(feature = "qbcl")]
    Qbt,

    /// Qubicle Construction Library, the `.qbcl` file.
    #[cfg(feature = "qbcl")]
    Qbcl,

    /// Voxel Max, the `.vmax` package directory.
    #[cfg(feature = "vmax")]
    VMax,

    /// Voxel Json, the `.voxj` and `.voxjz` documents.
    #[cfg(feature = "voxj")]
    Voxj,
}

impl ReadFormat {
    /// Every enabled format, in declaration order.
    pub const ALL: &'static [ReadFormat] = &[
        #[cfg(feature = "goxl")]
        ReadFormat::Goxl,
        #[cfg(feature = "mvox")]
        ReadFormat::MVox,
        #[cfg(feature = "qbcl")]
        ReadFormat::Qb,
        #[cfg(feature = "qbcl")]
        ReadFormat::Qbt,
        #[cfg(feature = "qbcl")]
        ReadFormat::Qbcl,
        #[cfg(feature = "vmax")]
        ReadFormat::VMax,
        #[cfg(feature = "voxj")]
        ReadFormat::Voxj,
    ];

    /// Runs `visitor` with this format's marker as its `F`.
    pub fn with<V: ReadFormatVisitor>(self, visitor: V) -> V::Output {
        match self {
            #[cfg(feature = "goxl")]
            ReadFormat::Goxl => visitor.visit::<Goxl>(),
            #[cfg(feature = "mvox")]
            ReadFormat::MVox => visitor.visit::<MVox>(),
            #[cfg(feature = "qbcl")]
            ReadFormat::Qb => visitor.visit::<Qb>(),
            #[cfg(feature = "qbcl")]
            ReadFormat::Qbt => visitor.visit::<Qbt>(),
            #[cfg(feature = "qbcl")]
            ReadFormat::Qbcl => visitor.visit::<Qbcl>(),
            #[cfg(feature = "vmax")]
            ReadFormat::VMax => visitor.visit::<VMax>(),
            #[cfg(feature = "voxj")]
            ReadFormat::Voxj => visitor.visit::<Voxj>(),
        }
    }

    /// The format a file extension implies, matched case-insensitively, or
    /// `None` when the extension implies no enabled format. `voxj` and
    /// `voxjz` both imply Voxel Json.
    pub fn from_extension(extension: &str) -> Option<Self> {
        let extension = extension.to_ascii_lowercase();

        Self::ALL
            .iter()
            .copied()
            .find(|format| format.with(Extensions).contains(&extension.as_str()))
    }

    /// The short lowercase name, as the format features spell it, with the
    /// three Qubicle formats told apart by extension.
    pub fn name(self) -> &'static str {
        self.with(Name)
    }

    /// Whether a document in this format is a package directory.
    pub fn is_package(self) -> bool {
        self.with(Package)
    }
}

/// A format's file extensions.
struct Extensions;

impl ReadFormatVisitor for Extensions {
    type Output = &'static [&'static str];

    fn visit<F: InstalledFormat>(self) -> Self::Output {
        F::EXTENSIONS
    }
}

/// A format's name.
struct Name;

impl ReadFormatVisitor for Name {
    type Output = &'static str;

    fn visit<F: InstalledFormat>(self) -> Self::Output {
        F::NAME
    }
}

/// Whether a format's document is a package.
struct Package;

impl ReadFormatVisitor for Package {
    type Output = bool;

    fn visit<F: InstalledFormat>(self) -> Self::Output {
        F::PACKAGE
    }
}

#[cfg(test)]
mod tests {
    use crate::ReadFormat;

    #[test]
    fn extensions_map_case_insensitively() {
        assert_eq!(ReadFormat::from_extension("vox"), Some(ReadFormat::MVox));

        assert_eq!(ReadFormat::from_extension("VOXJZ"), Some(ReadFormat::Voxj));

        assert_eq!(ReadFormat::from_extension("Qbt"), Some(ReadFormat::Qbt));

        assert_eq!(ReadFormat::from_extension("obj"), None);
    }

    #[test]
    fn only_the_vmax_package_is_a_directory() {
        assert!(ReadFormat::VMax.is_package());

        assert!(!ReadFormat::Voxj.is_package());

        assert_eq!(ReadFormat::Qbcl.name(), "qbcl");
    }
}
