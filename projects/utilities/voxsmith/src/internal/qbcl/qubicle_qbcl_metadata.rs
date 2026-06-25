use serde::{Deserialize, Serialize};

/// The seven free-text metadata strings in a `.qbcl` header, preserved verbatim
/// in the `qubicle-qbcl` ext. They have no native voxcore home.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct QubicleQbclMetadata {
    /// Document title.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,

    /// Document description.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,

    /// Tags.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tags: String,

    /// Author.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub author: String,

    /// Company.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub company: String,

    /// Website.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub website: String,

    /// Copyright notice.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub copyright: String,
}
