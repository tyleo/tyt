#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// The seven free-text metadata strings in a `.qbcl` header, preserved verbatim
/// in the `qubicle-qbcl` ext. They have no native voxcore home.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct QubicleQbclMetadata {
    /// Document title.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "String::is_empty")
    )]
    pub title: String,

    /// Document description.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "String::is_empty")
    )]
    pub description: String,

    /// Tags.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "String::is_empty")
    )]
    pub tags: String,

    /// Author.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "String::is_empty")
    )]
    pub author: String,

    /// Company.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "String::is_empty")
    )]
    pub company: String,

    /// Website.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "String::is_empty")
    )]
    pub website: String,

    /// Copyright notice.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "String::is_empty")
    )]
    pub copyright: String,
}
