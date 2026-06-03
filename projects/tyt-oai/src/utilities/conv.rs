use crate::Turn;

/// The contents of an `oai.img.json` (OpenAI image conversation) file.
///
/// Each entry in `conversations` is an independent conversation, stored as an
/// ordered list of [`Turn`]s. `next_image_id` is shared across all
/// conversations and names the next generated image (e.g. `3.png`).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "impl", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "impl", serde(rename_all = "camelCase"))]
pub struct Conv {
    /// Every conversation stored in the file.
    pub conversations: Vec<Vec<Turn>>,

    /// The id of the next image, used as its file name.
    pub next_image_id: u64,
}

impl Default for Conv {
    fn default() -> Self {
        Conv {
            conversations: Vec::new(),
            next_image_id: 1,
        }
    }
}
