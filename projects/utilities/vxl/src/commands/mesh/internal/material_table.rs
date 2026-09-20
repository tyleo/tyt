use crate::{Error, Result};
use branded_id::IdVec;
use meshdoc::BMeshMaterial;
use std::collections::BTreeMap;
use voxsmith::operations::mesh::MaterialRecord;

/// The materials the flags fill by index. `--material-count` declares the
/// count outright. Under it an index at or above the count errors, and an
/// unmentioned index below it emits an empty placeholder. Without it the
/// count derives as the highest mentioned index plus one, and a skipped index
/// errors.
pub(crate) struct MaterialTable {
    declared_count: Option<u32>,
    records: BTreeMap<u32, MaterialRecord>,
}

impl MaterialTable {
    pub(crate) fn new(declared_count: Option<u32>) -> Self {
        MaterialTable {
            declared_count,
            records: BTreeMap::new(),
        }
    }

    /// The record at `index`, which `flag` mentions, created empty on its
    /// first mention.
    pub(crate) fn material(&mut self, flag: &str, index: u32) -> Result<&mut MaterialRecord> {
        if let Some(count) = self.declared_count
            && index >= count
        {
            return Err(Error::usage(format!(
                "{flag} mentions material {index}, but --material-count {count} declares {}",
                match count {
                    0 => "no materials".to_owned(),
                    1 => "material 0 alone".to_owned(),
                    _ => format!("materials 0 to {}", count - 1),
                }
            )));
        }

        Ok(self.records.entry(index).or_default())
    }

    /// The table by id, every index below the count filled.
    pub(crate) fn finish(mut self) -> Result<IdVec<BMeshMaterial, MaterialRecord>> {
        let count = match self.declared_count {
            Some(count) => count,
            None => {
                let mentioned: Vec<u32> = self.records.keys().copied().collect();

                for (expected, index) in (0..).zip(mentioned) {
                    if index == expected {
                        continue;
                    }

                    return Err(Error::usage(format!(
                        "material {index} is mentioned but material {expected} is not. The derived \
                         count skips no index, so mention it or declare --material-count"
                    )));
                }

                self.records.keys().next_back().map_or(0, |last| last + 1)
            }
        };

        Ok(IdVec::from_vec(
            (0..count)
                .map(|index| self.records.remove(&index).unwrap_or_default())
                .collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::MaterialTable;

    #[test]
    fn the_count_derives_from_the_highest_mention() {
        let mut table = MaterialTable::new(None);
        table.material("--material-name", 2).unwrap().name = Some("c".to_owned());
        table.material("--material-name", 0).unwrap();
        table.material("--material-name", 1).unwrap();

        let materials = table.finish().unwrap();
        assert_eq!(materials.len(), 3);
        assert_eq!(materials.as_slice()[2].name.as_deref(), Some("c"));
    }

    #[test]
    fn no_mention_derives_no_materials() {
        assert!(MaterialTable::new(None).finish().unwrap().is_empty());
    }

    #[test]
    fn a_skipped_index_errors_without_a_declared_count() {
        let mut table = MaterialTable::new(None);
        table.material("--material-name", 1).unwrap();

        assert!(table.finish().is_err());
    }

    #[test]
    fn a_declared_count_fills_placeholders_and_caps_the_indices() {
        let mut table = MaterialTable::new(Some(3));
        table.material("--material-name", 1).unwrap();
        assert!(table.material("--material-name", 3).is_err());

        let materials = table.finish().unwrap();
        assert_eq!(materials.len(), 3);
        assert_eq!(materials.as_slice()[0].name, None);
    }

    #[test]
    fn a_declared_count_of_zero_admits_no_index() {
        let mut table = MaterialTable::new(Some(0));
        assert!(table.material("--material-name", 0).is_err());
        assert!(table.finish().unwrap().is_empty());
    }
}
