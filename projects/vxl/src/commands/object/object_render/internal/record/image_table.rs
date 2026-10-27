use crate::{
    Error, Result,
    commands::{ImageElements, ImageEntry, ImageRecord},
};
use std::collections::BTreeMap;

/// The images the flags fill by name. An image's name can suffix a file, so
/// an empty name or one holding a path separator errors.
#[derive(Debug, Default)]
pub struct ImageTable {
    images: BTreeMap<String, ImageElements>,
}

impl ImageTable {
    /// The elements of the image `name`, which `flag` mentions, created
    /// empty on the first mention.
    pub(crate) fn image(&mut self, flag: &str, name: &str) -> Result<&mut ImageElements> {
        check_file_suffix(flag, name)?;

        Ok(self
            .images
            .entry(name.to_owned())
            .or_insert_with(|| ImageElements::new(name.to_owned())))
    }

    /// Whether no flag mentions an image.
    pub(crate) fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// The images in name order, each flag element standing and `entries`,
    /// the profile stack's images, filling the rest.
    pub(crate) fn finish(
        mut self,
        entries: &BTreeMap<String, ImageEntry>,
    ) -> Result<Vec<ImageRecord>> {
        for name in entries.keys() {
            check_file_suffix("the profile stack", name)?;

            self.images
                .entry(name.clone())
                .or_insert_with(|| ImageElements::new(name.clone()));
        }

        self.images
            .into_iter()
            .map(|(name, elements)| elements.finish(entries.get(&name)))
            .collect()
    }
}

/// Errors unless `name`, which `origin` gives, can suffix a file: non-empty
/// and free of path separators.
fn check_file_suffix(origin: &str, name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Error::usage(format!(
            "{origin} gives an image an empty name, and the name suffixes the image's file"
        )));
    }

    if name.contains('/') || name.contains('\\') {
        return Err(Error::usage(format!(
            "{origin} names the image `{name}`, and the name suffixes the image's file, so it \
             cannot contain a path separator"
        )));
    }

    Ok(())
}
