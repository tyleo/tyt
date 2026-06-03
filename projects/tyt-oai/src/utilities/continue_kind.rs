use clap::ValueEnum;

/// How `tyt oai img` continues an existing conversation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum ContinueKind {
    /// Continue the most recent conversation in place via `previous_response_id`,
    /// appending the new turns to its existing subarray. Fails if that response
    /// is no longer cached by OpenAI.
    #[default]
    PreviousResponseId,

    /// Start a new conversation seeded with the full prior history, including
    /// every text turn and every image.
    AllImagesAllText,

    /// Start a new conversation seeded with every prior text turn plus the final
    /// image only.
    LastImageAllText,

    /// Start a new conversation seeded with only the final image and the new
    /// message; prior text is dropped.
    LastImageOnly,

    /// Start a new conversation appended as a fresh subarray, reusing no prior
    /// context or system prompt. `--system-prompt` still applies.
    NewConversation,
}
