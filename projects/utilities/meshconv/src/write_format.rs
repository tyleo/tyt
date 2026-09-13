#[cfg(feature = "gltf")]
use crate::gltf::{Gltf, GltfWriteFormat};
use crate::{InstalledFormat, ReadFormat, ReadFormatVisitor, WriteFormatVisitor};

/// A write target: the format and its writer options, one variant per
/// enabled format feature. Each variant stands for a
/// [`Format`](crate::Format) marker, and [`with`](WriteFormat::with) hands
/// that marker and the options to a visitor.
#[derive(Clone, Debug, PartialEq)]
pub enum WriteFormat {
    /// glTF 2.0, a `.glb` or `.gltf` document.
    #[cfg(feature = "gltf")]
    Gltf(GltfWriteFormat),
}

impl WriteFormat {
    /// Runs `visitor` with this target's marker as its `F` and its writer
    /// options.
    pub fn with<V: WriteFormatVisitor>(&self, visitor: V) -> V::Output {
        match self {
            #[cfg(feature = "gltf")]
            WriteFormat::Gltf(options) => visitor.visit::<Gltf>(options),
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
        gltf::{GltfContainer, GltfWriteFormat, GltfWriteOptions},
    };

    #[test]
    fn a_read_format_defaults_its_writer() {
        let format = WriteFormat::from(ReadFormat::Gltf);

        assert_eq!(
            format,
            WriteFormat::Gltf(GltfWriteFormat {
                container: GltfContainer::Glb,
                options: GltfWriteOptions::default(),
            })
        );

        assert_eq!(format.extension(), "glb");

        assert_eq!(format.read_format(), ReadFormat::Gltf);
    }

    #[test]
    fn the_json_container_takes_its_extension() {
        let format = WriteFormat::Gltf(GltfWriteFormat {
            container: GltfContainer::Gltf,
            options: GltfWriteOptions::default(),
        });

        assert_eq!(format.extension(), "gltf");
    }
}
