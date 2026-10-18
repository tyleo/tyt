use crate::{
    Result,
    commands::{ViewElements, ViewEntry},
};
use branded_id::IdVec;
use std::collections::BTreeMap;
use voxsmith::operations::object::{BRenderView, ViewRecord};

/// The views the flags fill by name.
#[derive(Debug, Default)]
pub struct ViewTable {
    views: BTreeMap<String, ViewElements>,

    // The flag that first mentions each view.
    first_flags: BTreeMap<String, String>,
}

impl ViewTable {
    /// The elements of the view `name`, which `flag` mentions, created empty
    /// on the first mention.
    pub(crate) fn view(&mut self, flag: &str, name: &str) -> Result<&mut ViewElements> {
        self.first_flags
            .entry(name.to_owned())
            .or_insert_with(|| flag.to_owned());

        Ok(self
            .views
            .entry(name.to_owned())
            .or_insert_with(|| ViewElements::new(name.to_owned())))
    }

    /// Whether no flag mentions a view.
    pub(crate) fn is_empty(&self) -> bool {
        self.views.is_empty()
    }

    /// Each view a flag mentions, by name, beside the flag that first
    /// mentions it.
    pub(crate) fn first_flags(&self) -> &BTreeMap<String, String> {
        &self.first_flags
    }

    /// The views by id in name order, each flag element standing and
    /// `entries`, the profile stack's views, filling the rest.
    pub(crate) fn finish(
        mut self,
        entries: &BTreeMap<String, ViewEntry>,
    ) -> Result<IdVec<BRenderView, ViewRecord>> {
        for name in entries.keys() {
            self.views
                .entry(name.clone())
                .or_insert_with(|| ViewElements::new(name.clone()));
        }

        self.views
            .into_iter()
            .map(|(name, elements)| elements.finish(entries.get(&name)))
            .collect::<Result<Vec<_>>>()
            .map(IdVec::from)
    }
}
