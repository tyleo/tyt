#[cfg(feature = "sdfj")]
use crate::sdfj::Sdfj;
use crate::{Format, ReadFormatVisitor};

/// An SDF document format a document is read as, one variant per enabled
/// format feature. Each variant stands for a [`Format`] marker, and
/// [`with`](ReadFormat::with) hands that marker to a visitor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReadFormat {
    /// SDF Json, the `.sdfj` and `.sdfjz` documents.
    #[cfg(feature = "sdfj")]
    Sdfj,
}

impl ReadFormat {
    /// Every enabled format, in declaration order.
    pub const ALL: &'static [ReadFormat] = &[
        #[cfg(feature = "sdfj")]
        ReadFormat::Sdfj,
    ];

    /// Runs `visitor` with this format's marker as its `F`.
    pub fn with<V: ReadFormatVisitor>(self, visitor: V) -> V::Output {
        match self {
            #[cfg(feature = "sdfj")]
            ReadFormat::Sdfj => visitor.visit::<Sdfj>(),
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
}

/// A format's file extensions.
struct Extensions;

impl ReadFormatVisitor for Extensions {
    type Output = &'static [&'static str];

    fn visit<F: Format>(self) -> Self::Output {
        F::EXTENSIONS
    }
}

/// A format's name.
struct Name;

impl ReadFormatVisitor for Name {
    type Output = &'static str;

    fn visit<F: Format>(self) -> Self::Output {
        F::NAME
    }
}

#[cfg(test)]
mod tests {
    use crate::ReadFormat;

    #[test]
    fn extensions_map_case_insensitively() {
        assert_eq!(ReadFormat::from_extension("sdfj"), Some(ReadFormat::Sdfj));

        assert_eq!(ReadFormat::from_extension("SDFJ"), Some(ReadFormat::Sdfj));

        assert_eq!(ReadFormat::from_extension("sdfjz"), Some(ReadFormat::Sdfj));

        assert_eq!(ReadFormat::from_extension("voxj"), None);

        assert_eq!(ReadFormat::Sdfj.name(), "sdfj");
    }
}
