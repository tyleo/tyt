use crate::Role;
use std::path::PathBuf;

/// A single message in a request to the model.
///
/// When `image` is set the message is sent as a multi-part user turn; otherwise
/// it is a plain text turn. When both `text` and `image` are present,
/// `image_first` controls their order within the turn.
#[derive(Clone, Debug)]
pub struct InputMessage {
    /// Who authored the message.
    pub role: Role,

    /// The text content, if any.
    pub text: Option<String>,

    /// A prior image to attach, referenced by file path, if any.
    pub image: Option<PathBuf>,

    /// When both `text` and `image` are present, whether the image precedes the
    /// text.
    pub image_first: bool,
}

impl InputMessage {
    /// Creates a text-only user message.
    pub fn user_text(text: impl Into<String>) -> Self {
        InputMessage {
            role: Role::User,
            text: Some(text.into()),
            image: None,
            image_first: false,
        }
    }

    /// Creates a text-only message authored by `role`.
    pub fn text(role: Role, text: impl Into<String>) -> Self {
        InputMessage {
            role,
            text: Some(text.into()),
            image: None,
            image_first: false,
        }
    }

    /// Creates a user message bundling an image with optional text.
    pub fn user_image(image: PathBuf, text: Option<String>, image_first: bool) -> Self {
        InputMessage {
            role: Role::User,
            text,
            image: Some(image),
            image_first,
        }
    }
}
