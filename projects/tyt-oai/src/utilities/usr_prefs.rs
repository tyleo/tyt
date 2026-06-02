/// User preferences for `tyt oai`, loaded from the `oai` section of
/// `.tytusrconfig`.
#[derive(Debug, Default)]
#[cfg_attr(feature = "impl", derive(serde::Deserialize))]
pub struct UsrPrefs {
    /// The OpenAI API key.
    #[cfg_attr(feature = "impl", serde(rename = "apiKey"))]
    pub api_key: Option<String>,
}
