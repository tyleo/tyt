#[cfg(not(feature = "ext"))]
use crate::Format;
#[cfg(feature = "ext")]
use crate::ext::FormatExt;

/// Every trait an installed format implements: [`Format`](crate::Format), and
/// [`FormatExt`](crate::ext::FormatExt) with the `ext` feature. The visitors
/// take it as their bound, so one dispatch serves both paths.
#[cfg(feature = "ext")]
pub trait InstalledFormat: FormatExt {}

#[cfg(feature = "ext")]
impl<F: FormatExt> InstalledFormat for F {}

/// Every trait an installed format implements: [`Format`](crate::Format), and
/// `FormatExt` with the `ext` feature. The visitors take it as their bound, so
/// one dispatch serves both paths.
#[cfg(not(feature = "ext"))]
pub trait InstalledFormat: Format {}

#[cfg(not(feature = "ext"))]
impl<F: Format> InstalledFormat for F {}
