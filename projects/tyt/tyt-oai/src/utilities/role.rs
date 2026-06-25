/// The author of a conversation turn or request message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "impl", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "impl", serde(rename_all = "lowercase"))]
pub enum Role {
    /// A message authored by the user.
    User,
    /// A message authored by the model.
    Assistant,
    /// A system prompt steering the model, prepended ahead of the conversation.
    System,
}
