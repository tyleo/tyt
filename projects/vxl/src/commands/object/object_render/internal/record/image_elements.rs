use crate::{
    CliValue, Error, Result,
    commands::{ImageEntry, ImageKind, ImageRecord},
};
use ty_math::TySrgbU8;

/// An image's elements as the flags set them. A flag setting an element twice
/// errors, and the profile stack's entry fills whatever no flag set.
#[derive(Debug)]
pub struct ImageElements {
    name: String,

    kind: Option<ImageKind>,

    view: Option<String>,

    views: Vec<String>,

    border_width: Option<u32>,

    border_color: Option<TySrgbU8>,
}

impl ImageElements {
    /// The empty elements of the image `name`.
    pub(crate) fn new(name: String) -> Self {
        ImageElements {
            name,
            kind: None,
            view: None,
            views: Vec::new(),
            border_width: None,
            border_color: None,
        }
    }

    /// Sets the kind, which `flag` gives.
    pub(crate) fn set_kind(&mut self, flag: &str, kind: ImageKind) -> Result<()> {
        claim(&mut self.kind, kind, flag, &self.name, "kind")
    }

    /// Sets the view a `view` image draws, which `flag` gives.
    pub(crate) fn set_view(&mut self, flag: &str, view: String) -> Result<()> {
        claim(&mut self.view, view, flag, &self.name, "view")
    }

    /// Adds a view to the end of a sheet's tiles.
    pub(crate) fn push_view(&mut self, view: String) {
        self.views.push(view);
    }

    /// Sets a sheet's border width in pixels, which `flag` gives.
    pub(crate) fn set_border_width(&mut self, flag: &str, width: u32) -> Result<()> {
        claim(
            &mut self.border_width,
            width,
            flag,
            &self.name,
            "border width",
        )
    }

    /// Sets a sheet's border color, which `flag` gives.
    pub(crate) fn set_border_color(&mut self, flag: &str, color: TySrgbU8) -> Result<()> {
        claim(
            &mut self.border_color,
            color,
            flag,
            &self.name,
            "border color",
        )
    }

    /// The image record the elements lower into over `entry`, the profile
    /// stack's image of the same name. A flag's element stands, and the entry
    /// fills the rest. Errors if an element belongs to the other kind, or a
    /// sheet lists no view or one view twice.
    pub(crate) fn finish(self, entry: Option<&ImageEntry>) -> Result<ImageRecord> {
        let name = self.name;

        let kind = self
            .kind
            .or(entry.and_then(|entry| entry.kind).map(|kind| kind.0))
            .unwrap_or(ImageKind::View);

        let view = self
            .view
            .or_else(|| entry.and_then(|entry| entry.view.clone()));

        let views = if self.views.is_empty() {
            entry.and_then(|entry| entry.views.clone())
        } else {
            Some(self.views)
        };

        let border_width = self
            .border_width
            .or(entry.and_then(|entry| entry.border_width));

        let border_color = self.border_color.or(entry
            .and_then(|entry| entry.border_color)
            .map(|color| color.0));

        match kind {
            ImageKind::Sheet => {
                if view.is_some() {
                    return Err(Error::usage(format!(
                        "sheet `{name}` sets one view, which a view image draws; a sheet lists \
                         its views with --image-views"
                    )));
                }

                let views = views.filter(|views| !views.is_empty()).ok_or_else(|| {
                    Error::usage(format!(
                        "sheet `{name}` lists no view; give it --image-views"
                    ))
                })?;

                for (position, view) in (0..).zip(&views) {
                    if views[..position].contains(view) {
                        return Err(Error::usage(format!(
                            "sheet `{name}` lists the view `{view}` twice"
                        )));
                    }
                }

                Ok(ImageRecord::Sheet {
                    name,
                    views,
                    border_width: border_width.unwrap_or(8),
                    border_color: border_color.unwrap_or(TySrgbU8::new(128, 128, 128)),
                })
            }

            ImageKind::View => {
                let sheet_only = [
                    (views.is_some(), "views"),
                    (border_width.is_some(), "a border width"),
                    (border_color.is_some(), "a border color"),
                ]
                .into_iter()
                .find_map(|(set, element)| set.then_some(element));

                if let Some(element) = sheet_only {
                    return Err(Error::usage(format!(
                        "image `{name}` sets {element}, which a sheet takes, and its kind is {}; \
                         give --image-kind {name} sheet",
                        kind.name()
                    )));
                }

                Ok(ImageRecord::View {
                    view: view.unwrap_or_else(|| name.clone()),
                    name,
                })
            }
        }
    }
}

/// Fills `slot` with `value`, the `element` of the image `name` that `flag`
/// gives. A slot filled already errors.
fn claim<T>(slot: &mut Option<T>, value: T, flag: &str, name: &str, element: &str) -> Result<()> {
    if slot.is_some() {
        return Err(Error::usage(format!(
            "{flag} sets image `{name}`'s {element}, which is set already"
        )));
    }

    *slot = Some(value);

    Ok(())
}
