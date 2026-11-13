use crate::{Error, Result};
use std::process::Command;

/// The system property that holds the start of the name of the scene to
/// view.
const VIEW: &str = "debug.voxrenderbench.view";

/// The system property that turns on the log of the view's pipeline
/// statistics.
const STATISTICS: &str = "debug.voxrenderbench.statistics";

/// The system property that turns on the gallery of models.
const MODELS: &str = "debug.voxrenderbench.models";

/// The system property that turns on the gallery of test models.
const TESTS: &str = "debug.voxrenderbench.tests";

/// The system property that holds text from the path of the gallery's first
/// model.
const MODEL: &str = "debug.voxrenderbench.model";

/// The system property that holds the seconds the gallery shows a model
/// before it steps to the next.
const TOUR: &str = "debug.voxrenderbench.tour";

/// The system property that holds the meters a second the view's wearer
/// walks.
const WALK: &str = "debug.voxrenderbench.walk";

/// The system property that holds the part of the eyes' recommended size
/// the swapchain takes along each side.
const SCALE: &str = "debug.voxrenderbench.scale";

/// The bench scene the app shows while [`VIEW`] is unset.
const DEFAULT_VIEW: &str = "S1";

/// What the app runs, from the system properties the README's steps set.
pub struct RunOptions {
    /// The start of the name of the bench scene the app shows while no
    /// gallery is on.
    pub view: String,

    /// Whether the app logs the driver's statistics for the view's
    /// pipelines.
    pub statistics: bool,

    /// Whether the app shows the gallery of the models in its files in
    /// place of a scene.
    pub models: bool,

    /// Whether the gallery shows the test models in the app's files in
    /// place of the models.
    pub tests: bool,

    /// Text the path of the gallery's first model holds.
    pub model: Option<String>,

    /// The seconds the gallery shows a model before it steps to the next
    /// with no button pressed.
    pub tour: Option<f64>,

    /// The meters a second the view's wearer walks down the scene and back
    /// with no step taken.
    pub walk: Option<f64>,

    /// The part of the eyes' recommended size the swapchain takes along
    /// each side, 1 when the property is unset.
    pub scale: f64,
}

impl RunOptions {
    /// Reads the properties. Errors unless each flag is unset, `true`, or
    /// `false`, the tour and the walk are each unset or a number over 0,
    /// and the scale is unset or over 0 and at most 1.
    pub fn read() -> Result<Self> {
        let text = |name| getprop(name).map(|value| (!value.is_empty()).then_some(value));

        let positive = |name| {
            text(name)?
                .map(|value| {
                    value
                        .parse()
                        .ok()
                        .filter(|&number: &f64| number.is_finite() && number > 0.0)
                        .ok_or(Error::PositiveProperty { name, value })
                })
                .transpose()
        };

        let scale = text(SCALE)?
            .map(|value| {
                value
                    .parse()
                    .ok()
                    .filter(|&scale: &f64| scale > 0.0 && scale <= 1.0)
                    .ok_or(Error::ScaleProperty { name: SCALE, value })
            })
            .transpose()?
            .unwrap_or(1.0);

        Ok(RunOptions {
            view: text(VIEW)?.unwrap_or_else(|| DEFAULT_VIEW.to_owned()),
            statistics: flag(STATISTICS, false)?,
            models: flag(MODELS, false)?,
            tests: flag(TESTS, false)?,
            model: text(MODEL)?,
            tour: positive(TOUR)?,
            walk: positive(WALK)?,
            scale,
        })
    }
}

/// The value of the system property `name`, or an empty string when the
/// property is unset.
fn getprop(name: &str) -> Result<String> {
    let output = Command::new("getprop")
        .arg(name)
        .output()
        .map_err(Error::Getprop)?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// The flag the system property `name` holds, or `unset` when the property
/// is unset.
fn flag(name: &'static str, unset: bool) -> Result<bool> {
    match getprop(name)?.as_str() {
        "" => Ok(unset),
        "true" => Ok(true),
        "false" => Ok(false),
        value => Err(Error::FlagProperty {
            name,
            value: value.to_owned(),
        }),
    }
}
