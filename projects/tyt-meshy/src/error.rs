use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};

/// An error from this crate.
#[derive(Debug)]
pub enum Error {
    /// An I/O error.
    IO(IOError),

    /// No Meshy API key was configured.
    ApiKeyNotConfigured,

    /// An option was combined with `--model-type lowpoly`, which ignores it.
    /// Holds the conflicting flag's name.
    LowpolyConflict(&'static str),

    /// A texturing option was given without `--texture`. Holds the flag's name.
    TextureOptionWithoutTexture(&'static str),

    /// More than one of `--texture-prompt`, `--texture-prompt-file`, and
    /// `--texture-image` was given.
    TexturePromptConflict,

    /// The texture prompt exceeded the 600-character maximum. Holds its length.
    TexturePromptTooLong(usize),

    /// A remesh option was given without `--remesh`. Holds the flag's name.
    RemeshOptionWithoutRemesh(&'static str),

    /// `--target-polycount` was combined with `--decimation-mode`.
    PolycountDecimationConflict,

    /// `--image-enhancement` was given for a model that does not support it.
    ImageEnhancementUnavailable,

    /// `--keep-lighting` was given for a model that does not support it.
    KeepLightingUnavailable,

    /// `--texture-quality hd` was given for a model that does not support it.
    HdTextureUnavailable,

    /// A polling option was given without `--poll`. Holds the flag's name.
    PollOptionWithoutPoll(&'static str),

    /// An input image used an extension Meshy does not accept. Holds the
    /// extension.
    UnsupportedImageFormat(String),

    /// The request could not be sent or its response could not be received.
    Http(String),

    /// Meshy returned an error response.
    Api(String),

    /// The Meshy response could not be interpreted.
    InvalidResponse(String),

    /// The Meshy task did not succeed. Holds its status and error message.
    TaskFailed(String, String),

    /// Polling timed out before the task completed. Holds the timeout, in
    /// seconds.
    PollTimeout(u64),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::IO(e) => e.fmt(f),
            Error::ApiKeyNotConfigured => f.write_str(
                "no Meshy API key configured; add a \"meshy\" section with an \"apiKey\" \
                 to a .tytusrconfig file at your git root",
            ),
            Error::LowpolyConflict(flag) => {
                write!(f, "{flag} cannot be combined with --model-type lowpoly")
            }
            Error::TextureOptionWithoutTexture(flag) => write!(f, "{flag} requires --texture"),
            Error::TexturePromptConflict => f.write_str(
                "--texture-prompt, --texture-prompt-file, and --texture-image are mutually \
                 exclusive; pass at most one",
            ),
            Error::TexturePromptTooLong(len) => {
                write!(
                    f,
                    "the texture prompt is {len} characters, but the maximum is 600"
                )
            }
            Error::RemeshOptionWithoutRemesh(flag) => write!(f, "{flag} requires --remesh"),
            Error::PolycountDecimationConflict => {
                f.write_str("--target-polycount cannot be combined with --decimation-mode")
            }
            Error::ImageEnhancementUnavailable => f.write_str(
                "--image-enhancement is only supported when --model is meshy-6 or latest",
            ),
            Error::KeepLightingUnavailable => {
                f.write_str("--keep-lighting is only supported when --model is meshy-6 or latest")
            }
            Error::HdTextureUnavailable => f.write_str(
                "--texture-quality hd is only supported when --model is meshy-6 or latest",
            ),
            Error::PollOptionWithoutPoll(flag) => write!(f, "{flag} requires --poll"),
            Error::UnsupportedImageFormat(ext) => write!(
                f,
                "unsupported image format \"{ext}\"; Meshy accepts png, jpg, and jpeg"
            ),
            Error::Http(e) => write!(f, "Meshy request failed: {e}"),
            Error::Api(e) => write!(f, "Meshy API error: {e}"),
            Error::InvalidResponse(e) => write!(f, "unexpected Meshy response: {e}"),
            Error::TaskFailed(status, message) => {
                write!(f, "the Meshy task failed with status {status}")?;
                if !message.is_empty() {
                    write!(f, ": {message}")?;
                }
                Ok(())
            }
            Error::PollTimeout(seconds) => write!(
                f,
                "timed out after {seconds}s waiting for the Meshy task to complete; its output \
                 was left \"pending\" and can be fetched later"
            ),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::IO(e) => Some(e),
            Error::ApiKeyNotConfigured
            | Error::LowpolyConflict(_)
            | Error::TextureOptionWithoutTexture(_)
            | Error::TexturePromptConflict
            | Error::TexturePromptTooLong(_)
            | Error::RemeshOptionWithoutRemesh(_)
            | Error::PolycountDecimationConflict
            | Error::ImageEnhancementUnavailable
            | Error::KeepLightingUnavailable
            | Error::HdTextureUnavailable
            | Error::PollOptionWithoutPoll(_)
            | Error::UnsupportedImageFormat(_)
            | Error::Http(_)
            | Error::Api(_)
            | Error::InvalidResponse(_)
            | Error::TaskFailed(..)
            | Error::PollTimeout(_) => None,
        }
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::IO(e)
    }
}
