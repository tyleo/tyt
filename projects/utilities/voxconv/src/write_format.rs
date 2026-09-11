#[cfg(feature = "goxl")]
use crate::goxl::Goxl;
#[cfg(feature = "mvox")]
use crate::mvox::MVox;
#[cfg(feature = "qbcl")]
use crate::qbcl::{Qb, Qbcl, Qbt};
#[cfg(feature = "vmax")]
use crate::vmax::{VMax, VMaxWriteOptions};
#[cfg(feature = "voxj")]
use crate::voxj::{Voxj, VoxjWriteFormat};
use crate::{InstalledFormat, ReadFormat, ReadFormatVisitor, WriteFormatVisitor};

/// A write target: the format and its writer options, one variant per
/// enabled format feature. Each variant stands for a
/// [`Format`](crate::Format) marker, and [`with`](WriteFormat::with) hands
/// that marker and the options to a visitor.
#[derive(Clone, Debug, PartialEq)]
pub enum WriteFormat {
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
    VMax(VMaxWriteOptions),

    /// Voxel Json, a `.voxj` or `.voxjz` document.
    #[cfg(feature = "voxj")]
    Voxj(VoxjWriteFormat),
}

impl WriteFormat {
    /// Runs `visitor` with this target's marker as its `F` and its writer
    /// options.
    pub fn with<V: WriteFormatVisitor>(&self, visitor: V) -> V::Output {
        match self {
            #[cfg(feature = "goxl")]
            WriteFormat::Goxl => visitor.visit::<Goxl>(&()),
            #[cfg(feature = "mvox")]
            WriteFormat::MVox => visitor.visit::<MVox>(&()),
            #[cfg(feature = "qbcl")]
            WriteFormat::Qb => visitor.visit::<Qb>(&()),
            #[cfg(feature = "qbcl")]
            WriteFormat::Qbt => visitor.visit::<Qbt>(&()),
            #[cfg(feature = "qbcl")]
            WriteFormat::Qbcl => visitor.visit::<Qbcl>(&()),
            #[cfg(feature = "vmax")]
            WriteFormat::VMax(options) => visitor.visit::<VMax>(options),
            #[cfg(feature = "voxj")]
            WriteFormat::Voxj(options) => visitor.visit::<Voxj>(options),
        }
    }

    /// The extension a document written in this format takes.
    pub fn extension(&self) -> &'static str {
        self.with(Extension)
    }

    /// The format a document written in this format reads back as.
    pub fn read_format(&self) -> ReadFormat {
        self.with(AsReadFormat)
    }
}

/// The write target for a format with default writer options.
impl From<ReadFormat> for WriteFormat {
    fn from(format: ReadFormat) -> Self {
        format.with(DefaultWriteFormat)
    }
}

/// The extension a write takes.
struct Extension;

impl WriteFormatVisitor for Extension {
    type Output = &'static str;

    fn visit<F: InstalledFormat>(self, options: &F::WriteOptions) -> Self::Output {
        F::extension(options)
    }
}

/// The read format of a write.
struct AsReadFormat;

impl WriteFormatVisitor for AsReadFormat {
    type Output = ReadFormat;

    fn visit<F: InstalledFormat>(self, _options: &F::WriteOptions) -> Self::Output {
        F::read_format()
    }
}

/// A format's write target with default options.
struct DefaultWriteFormat;

impl ReadFormatVisitor for DefaultWriteFormat {
    type Output = WriteFormat;

    fn visit<F: InstalledFormat>(self) -> Self::Output {
        F::write_format(F::WriteOptions::default())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ReadFormat, WriteFormat,
        voxj::{VoxjSerialization, VoxjWriteFormat, VoxjWriteOptions},
    };

    #[test]
    fn a_read_format_defaults_its_writer() {
        let format = WriteFormat::from(ReadFormat::Voxj);

        assert_eq!(
            format,
            WriteFormat::Voxj(VoxjWriteFormat {
                serialization: VoxjSerialization::Compact,
                options: VoxjWriteOptions::default(),
            })
        );

        assert_eq!(format.extension(), "voxj");

        assert_eq!(format.read_format(), ReadFormat::Voxj);
    }

    #[test]
    fn the_zip_serialization_takes_its_extension() {
        let format = WriteFormat::Voxj(VoxjWriteFormat {
            serialization: VoxjSerialization::Zip,
            options: VoxjWriteOptions::default(),
        });

        assert_eq!(format.extension(), "voxjz");
    }
}
