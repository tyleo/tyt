#[cfg(feature = "gltf")]
use crate::gltf::Gltf;
use crate::{InstalledFormat, ReadFormatVisitor, Result};

/// A mesh document format a document is read as, one variant per enabled
/// format feature. Each variant stands for a [`Format`](crate::Format)
/// marker, and [`with`](ReadFormat::with) hands that marker to a visitor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReadFormat {
    /// glTF 2.0, the `.glb` and `.gltf` files.
    #[cfg(feature = "gltf")]
    Gltf,
}

impl ReadFormat {
    /// Every enabled format, in declaration order.
    pub const ALL: &'static [ReadFormat] = &[
        #[cfg(feature = "gltf")]
        ReadFormat::Gltf,
    ];

    /// Runs `visitor` with this format's marker as its `F`.
    pub fn with<V: ReadFormatVisitor>(self, visitor: V) -> V::Output {
        match self {
            #[cfg(feature = "gltf")]
            ReadFormat::Gltf => visitor.visit::<Gltf>(),
        }
    }

    /// The format a file extension implies, matched case-insensitively, or
    /// `None` when the extension implies no enabled format.
    pub fn from_extension(extension: &str) -> Option<Self> {
        let extension = extension.to_ascii_lowercase();

        Self::ALL
            .iter()
            .copied()
            .find(|format| format.with(Extensions).contains(&extension.as_str()))
    }

    /// The short lowercase name, matching the format's feature.
    pub fn name(self) -> &'static str {
        self.with(Name)
    }

    /// Whether a document in this format is a package directory.
    pub fn is_package(self) -> bool {
        self.with(Package)
    }

    /// The relative paths of the loose files `primary` references beside
    /// it.
    pub fn loose_paths(self, primary: &[u8]) -> Result<Vec<String>> {
        self.with(LoosePaths(primary))
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

/// The loose files a primary references.
struct LoosePaths<'a>(&'a [u8]);

impl ReadFormatVisitor for LoosePaths<'_> {
    type Output = Result<Vec<String>>;

    fn visit<F: InstalledFormat>(self) -> Self::Output {
        F::loose_paths(self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::ReadFormat;

    #[test]
    fn extensions_map_case_insensitively() {
        assert_eq!(ReadFormat::from_extension("glb"), Some(ReadFormat::Gltf));

        assert_eq!(ReadFormat::from_extension("GLTF"), Some(ReadFormat::Gltf));

        assert_eq!(ReadFormat::from_extension("obj"), None);
    }

    #[test]
    fn gltf_is_not_a_package() {
        assert!(!ReadFormat::Gltf.is_package());

        assert_eq!(ReadFormat::Gltf.name(), "gltf");
    }
}
