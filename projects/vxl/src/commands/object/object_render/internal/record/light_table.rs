use crate::{
    Error, Result,
    commands::{LightElements, LightEntry, LightKind},
};
use branded_id::IdVec;
use std::collections::BTreeMap;
use voxsmith::operations::object::{BRenderLight, LightRecord};

/// The lights the flags fill by index: the `--light` declarations, else the
/// rig the profile stack supplies. An index at or above the count errors.
pub struct LightTable {
    lights: Vec<LightElements>,
}

impl LightTable {
    /// The rig the `--light` declarations `declared` build, each an index
    /// with its kind. An index declared twice or one skipped errors.
    pub(crate) fn declared(declared: &[(u32, LightKind)]) -> Result<Self> {
        let mut kinds = BTreeMap::new();

        for &(index, kind) in declared {
            if kinds.insert(index, kind).is_some() {
                return Err(Error::usage(format!(
                    "--light declares light {index} twice"
                )));
            }
        }

        for (expected, &index) in (0..).zip(kinds.keys()) {
            if index != expected {
                return Err(Error::usage(format!(
                    "--light declares light {index} but not light {expected}, and a rig skips \
                     no index"
                )));
            }
        }

        Ok(LightTable {
            lights: kinds
                .into_iter()
                .map(|(index, kind)| LightElements::declared(index, kind))
                .collect(),
        })
    }

    /// The rig `entries` supplies.
    pub(crate) fn from_rig(entries: &[LightEntry]) -> Self {
        LightTable {
            lights: (0..)
                .zip(entries)
                .map(|(index, entry)| LightElements::from_entry(index, entry.clone()))
                .collect(),
        }
    }

    /// The elements of light `index`, which `flag` refers to.
    pub(crate) fn light(&mut self, flag: &str, index: u32) -> Result<&mut LightElements> {
        let count = self.lights.len();

        self.lights.get_mut(index as usize).ok_or_else(|| {
            Error::usage(format!(
                "{flag} names light {index}, but the rig holds {}",
                match count {
                    0 => "no lights".to_owned(),
                    1 => "light 0 alone".to_owned(),
                    _ => format!("lights 0 to {}", count - 1),
                }
            ))
        })
    }

    /// The lights by id.
    pub(crate) fn finish(self) -> Result<IdVec<BRenderLight, LightRecord>> {
        self.lights
            .into_iter()
            .map(LightElements::finish)
            .collect::<Result<Vec<_>>>()
            .map(IdVec::from)
    }
}
