use crate::ReadFormat;
#[cfg(feature = "vmax")]
use crate::vmax::VMaxWriteOptions;
#[cfg(feature = "voxj")]
use crate::voxj::{VoxjSerialization, VoxjWriteOptions};

/// A write target: the format and its writer options, one variant per
/// enabled format feature.
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
    Voxj {
        serialization: VoxjSerialization,
        options: VoxjWriteOptions,
    },
}

impl WriteFormat {
    /// The extension a document written in this format takes.
    pub fn extension(&self) -> &'static str {
        match self {
            #[cfg(feature = "goxl")]
            WriteFormat::Goxl => "gox",
            #[cfg(feature = "mvox")]
            WriteFormat::MVox => "vox",
            #[cfg(feature = "qbcl")]
            WriteFormat::Qb => "qb",
            #[cfg(feature = "qbcl")]
            WriteFormat::Qbt => "qbt",
            #[cfg(feature = "qbcl")]
            WriteFormat::Qbcl => "qbcl",
            #[cfg(feature = "vmax")]
            WriteFormat::VMax(_) => "vmax",
            #[cfg(feature = "voxj")]
            WriteFormat::Voxj { serialization, .. } => serialization.extension(),
        }
    }

    /// The format a document written in this format reads back as.
    pub fn read_format(&self) -> ReadFormat {
        match self {
            #[cfg(feature = "goxl")]
            WriteFormat::Goxl => ReadFormat::Goxl,
            #[cfg(feature = "mvox")]
            WriteFormat::MVox => ReadFormat::MVox,
            #[cfg(feature = "qbcl")]
            WriteFormat::Qb => ReadFormat::Qb,
            #[cfg(feature = "qbcl")]
            WriteFormat::Qbt => ReadFormat::Qbt,
            #[cfg(feature = "qbcl")]
            WriteFormat::Qbcl => ReadFormat::Qbcl,
            #[cfg(feature = "vmax")]
            WriteFormat::VMax(_) => ReadFormat::VMax,
            #[cfg(feature = "voxj")]
            WriteFormat::Voxj { .. } => ReadFormat::Voxj,
        }
    }
}

/// The write target for a format with default writer options.
impl From<ReadFormat> for WriteFormat {
    fn from(format: ReadFormat) -> Self {
        match format {
            #[cfg(feature = "goxl")]
            ReadFormat::Goxl => WriteFormat::Goxl,
            #[cfg(feature = "mvox")]
            ReadFormat::MVox => WriteFormat::MVox,
            #[cfg(feature = "qbcl")]
            ReadFormat::Qb => WriteFormat::Qb,
            #[cfg(feature = "qbcl")]
            ReadFormat::Qbt => WriteFormat::Qbt,
            #[cfg(feature = "qbcl")]
            ReadFormat::Qbcl => WriteFormat::Qbcl,
            #[cfg(feature = "vmax")]
            ReadFormat::VMax => WriteFormat::VMax(VMaxWriteOptions::default()),
            #[cfg(feature = "voxj")]
            ReadFormat::Voxj => WriteFormat::Voxj {
                serialization: VoxjSerialization::default(),
                options: VoxjWriteOptions::default(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ReadFormat, WriteFormat,
        voxj::{VoxjSerialization, VoxjWriteOptions},
    };

    #[test]
    fn a_read_format_defaults_its_writer() {
        let format = WriteFormat::from(ReadFormat::Voxj);

        assert_eq!(
            format,
            WriteFormat::Voxj {
                serialization: VoxjSerialization::Compact,
                options: VoxjWriteOptions::default(),
            }
        );

        assert_eq!(format.extension(), "voxj");

        assert_eq!(format.read_format(), ReadFormat::Voxj);
    }

    #[test]
    fn the_zip_serialization_takes_its_extension() {
        let format = WriteFormat::Voxj {
            serialization: VoxjSerialization::Zip,
            options: VoxjWriteOptions::default(),
        };

        assert_eq!(format.extension(), "voxjz");
    }
}
