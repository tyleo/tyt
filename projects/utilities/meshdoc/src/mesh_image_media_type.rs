use std::fmt::{Display, Formatter, Result as FmtResult};

/// The encoding of a [`MeshImage`](crate::MeshImage)'s bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MeshImageMediaType {
    /// A PNG, `image/png`.
    Png,

    /// A JPEG, `image/jpeg`.
    Jpeg,
}

impl MeshImageMediaType {
    /// The media type string.
    pub fn as_str(self) -> &'static str {
        match self {
            MeshImageMediaType::Png => "image/png",
            MeshImageMediaType::Jpeg => "image/jpeg",
        }
    }

    /// The media type a string names, or `None` for any other string.
    pub fn from_media_type(media_type: &str) -> Option<Self> {
        match media_type {
            "image/png" => Some(MeshImageMediaType::Png),
            "image/jpeg" => Some(MeshImageMediaType::Jpeg),
            _ => None,
        }
    }

    /// The file extension of an image of this type, without the dot.
    pub fn extension(self) -> &'static str {
        match self {
            MeshImageMediaType::Png => "png",
            MeshImageMediaType::Jpeg => "jpg",
        }
    }

    /// Whether `bytes` start with this type's file signature.
    pub fn matches(self, bytes: &[u8]) -> bool {
        match self {
            MeshImageMediaType::Png => {
                bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
            }
            MeshImageMediaType::Jpeg => bytes.starts_with(&[0xFF, 0xD8, 0xFF]),
        }
    }
}

impl Display for MeshImageMediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use crate::MeshImageMediaType;

    #[test]
    fn signatures_match_their_type_only() {
        let png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0];
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0];

        assert!(MeshImageMediaType::Png.matches(&png));
        assert!(!MeshImageMediaType::Png.matches(&jpeg));
        assert!(MeshImageMediaType::Jpeg.matches(&jpeg));
        assert!(!MeshImageMediaType::Jpeg.matches(&png));
        assert!(!MeshImageMediaType::Jpeg.matches(&[]));
    }

    #[test]
    fn strings_round_trip() {
        for media_type in [MeshImageMediaType::Png, MeshImageMediaType::Jpeg] {
            assert_eq!(
                MeshImageMediaType::from_media_type(media_type.as_str()),
                Some(media_type)
            );
        }

        assert_eq!(MeshImageMediaType::from_media_type("image/webp"), None);
    }
}
