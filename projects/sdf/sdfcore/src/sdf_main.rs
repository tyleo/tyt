use crate::{BSdfFont, Result, SdfFontData, SdfState};
use branded_id::{IdSlice, IdVec};

/// A model whose state passed [`SdfState::validate`]. The model never edits
/// the state. The font data starts unloaded.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SdfMain {
    state: SdfState,

    font_data: Option<IdVec<BSdfFont, SdfFontData>>,
}

impl SdfMain {
    /// The model over `state`. Errors when `state` fails validation.
    pub fn new(state: SdfState) -> Result<Self> {
        state.validate()?;

        Ok(Self {
            state,
            font_data: None,
        })
    }

    /// The model with `font_data` loaded, one entry per font of the state.
    ///
    /// # Panics
    ///
    /// When `font_data` and the state's fonts differ in length.
    pub fn with_font_data(self, font_data: IdVec<BSdfFont, SdfFontData>) -> Self {
        assert_eq!(
            font_data.len(),
            self.state.fonts.len(),
            "each of the model's fonts comes with its data"
        );

        Self {
            font_data: Some(font_data),
            ..self
        }
    }

    /// The model's tables.
    pub fn state(&self) -> &SdfState {
        &self.state
    }

    /// The data of each of the model's fonts, or `None` when the fonts are
    /// unloaded.
    pub fn font_data(&self) -> Option<&IdSlice<BSdfFont, SdfFontData>> {
        self.font_data.as_deref()
    }

    /// The model's tables, taken out of the model.
    pub fn into_state(self) -> SdfState {
        self.state
    }
}
