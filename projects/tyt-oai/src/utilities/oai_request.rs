use crate::InputMessage;

/// A request to the OpenAI Responses API to continue an image conversation.
#[derive(Clone, Debug)]
pub struct OaiRequest {
    /// The model to send the request to (e.g. `gpt-5.1`).
    pub model: String,

    /// The id of the response to continue from, when continuing in place.
    pub previous_response_id: Option<String>,

    /// The messages making up the request input.
    pub input: Vec<InputMessage>,

    /// Whether to enable the `image_generation` tool.
    pub generate_image: bool,
}
