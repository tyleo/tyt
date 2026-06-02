/// The result of an OpenAI image-generation request.
#[derive(Clone, Debug)]
pub struct OaiResponse {
    /// The assistant's text reply.
    pub text: String,

    /// The decoded bytes of the generated image, if one was produced.
    pub image_png: Option<Vec<u8>>,

    /// The server-side response id, used to continue the conversation later.
    pub response_id: String,
}
