/// Preferences for `tyt oai img`, read from the `oai.img` section of
/// `.tytconfig` and `.tytusrconfig`.
#[derive(Debug, Default)]
#[cfg_attr(
    feature = "impl",
    derive(serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct ImgPrefs {
    /// The directory `--system-prompt` files are resolved against, interpreted
    /// relative to the git root when not absolute.
    pub system_prompts_dir: Option<String>,
}
