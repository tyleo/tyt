use crate::{Error, Result};
use branded_id::IdVec;
use meshdoc::BMeshMaterial;
use std::collections::BTreeMap;
use voxsmith::operations::mesh::MaterialRecord;

/// The materials the flags fill by index. A declared count, from
/// `--material-count` or a profile's materials list, caps the indices: one at
/// or above it errors, and an unmentioned index below it emits an empty
/// placeholder. Without one the count derives as the highest mentioned index
/// plus one, and a skipped index errors.
pub(crate) struct MaterialTable {
    declared: Option<(u32, String)>,
    records: BTreeMap<u32, MaterialRecord>,
}

impl MaterialTable {
    /// A table of `count` materials, which `declared_by` declares.
    pub(crate) fn declared(count: u32, declared_by: String) -> Self {
        MaterialTable {
            declared: Some((count, declared_by)),
            records: BTreeMap::new(),
        }
    }

    /// A table whose count derives from the mentions.
    pub(crate) fn derived() -> Self {
        MaterialTable {
            declared: None,
            records: BTreeMap::new(),
        }
    }

    /// The record at `index`, which `origin` mentions, created empty on its
    /// first mention.
    pub(crate) fn material(&mut self, origin: &str, index: u32) -> Result<&mut MaterialRecord> {
        if let Some((count, declared_by)) = &self.declared
            && index >= *count
        {
            return Err(Error::usage(format!(
                "{origin} mentions material {index}, but {declared_by} declares {}",
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
        let count = match &self.declared {
            Some((count, _)) => *count,

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
        let mut table = MaterialTable::derived();
        table.material("--material-name", 2).unwrap().name = Some("c".to_owned());
        table.material("--material-name", 0).unwrap();
        table.material("--material-name", 1).unwrap();

        let materials = table.finish().unwrap();
        assert_eq!(materials.len(), 3);
        assert_eq!(materials.as_slice()[2].name.as_deref(), Some("c"));
    }

    #[test]
    fn no_mention_derives_no_materials() {
        assert!(MaterialTable::derived().finish().unwrap().is_empty());
    }

    #[test]
    fn a_skipped_index_errors_without_a_declared_count() {
        let mut table = MaterialTable::derived();
        table.material("--material-name", 1).unwrap();

        assert!(table.finish().is_err());
    }

    #[test]
    fn a_declared_count_fills_placeholders_and_caps_the_indices() {
        let mut table = MaterialTable::declared(3, "--material-count 3".to_owned());
        table.material("--material-name", 1).unwrap();

        let error = table
            .material("--material-name", 3)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("--material-count 3 declares materials 0 to 2"),
            "{error}"
        );

        let materials = table.finish().unwrap();
        assert_eq!(materials.len(), 3);
        assert_eq!(materials.as_slice()[0].name, None);
    }

    #[test]
    fn a_declared_count_of_zero_admits_no_index() {
        let mut table = MaterialTable::declared(0, "the profile `geometry`".to_owned());

        let error = table
            .material("--material-name", 0)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("the profile `geometry` declares no materials"),
            "{error}"
        );
        assert!(table.finish().unwrap().is_empty());
    }
}
