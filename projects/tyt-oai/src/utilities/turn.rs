use crate::Role;

/// A single turn in a stored conversation.
///
/// User turns carry a [`Role`] and `content`, and may carry an `image`: either a
/// "previous conversation" image fed back as context by a reconstruction, or the
/// image an image-only reconstruction attaches to the new message. Assistant
/// turns carry the `image` they generated (omitted when `--no-gen` was used),
/// its `revisedPrompt`, and the `responseId` that lets the next request continue
/// from this point.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "impl", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "impl", serde(rename_all = "camelCase"))]
pub struct Turn {
    /// Who authored the turn.
    pub role: Role,

    /// The text content of the turn.
    pub content: String,

    /// The file name of an image attached to this turn, relative to the
    /// conversation directory: the image an assistant turn generated, or a prior
    /// image carried onto a user turn by a reconstruction.
    #[cfg_attr(
        feature = "impl",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub image: Option<String>,

    /// The prompt OpenAI revised the generated image from, when it supplied
    /// one. Present only on assistant turns that produced an image.
    #[cfg_attr(
        feature = "impl",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub revised_prompt: Option<String>,

    /// The OpenAI server-side response id of an assistant turn.
    #[cfg_attr(
        feature = "impl",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub response_id: Option<String>,
}
