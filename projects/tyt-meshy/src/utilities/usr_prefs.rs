/// Preferences for `tyt meshy`, loaded from the `meshy` section of
/// `.tytusrconfig`.
#[derive(Debug, Default)]
#[cfg_attr(feature = "impl", derive(serde::Deserialize))]
pub struct UsrPrefs {
    /// The Meshy API key.
    #[cfg_attr(feature = "impl", serde(rename = "apiKey"))]
    pub api_key: Option<String>,
}
