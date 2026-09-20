use crate::{Error, Result};
use branded_id::{IdVec, U32Id};
use meshdoc::BMeshPrimitive;
use voxsmith::operations::mesh::PrimitiveRecord;

/// The primitives the flags fill by index: the `--primitive` declarations in
/// flag order, else the implicit whole-mesh primitive. An index at or above
/// the count errors.
pub(crate) struct PrimitiveTable {
    records: Vec<PrimitiveRecord>,
}

impl PrimitiveTable {
    /// The table of `declared`, or of the implicit primitive when nothing is
    /// declared. The implicit primitive takes every face and draws with
    /// material 0 when `material_count` is not zero, else with no material.
    pub(crate) fn new(declared: Vec<PrimitiveRecord>, material_count: u32) -> Self {
        if !declared.is_empty() {
            return PrimitiveTable { records: declared };
        }

        PrimitiveTable {
            records: vec![PrimitiveRecord {
                material_id: (material_count > 0).then(|| U32Id::from_u32(0)),
                select: "true".to_owned(),
                name: None,
                normal: true,
                uv_streams: None,
                attributes: Vec::new(),
            }],
        }
    }

    /// The record at `index`, which `flag` refers to.
    pub(crate) fn primitive(&mut self, flag: &str, index: u32) -> Result<&mut PrimitiveRecord> {
        let count = self.records.len();

        self.records.get_mut(index as usize).ok_or_else(|| {
            Error::usage(format!(
                "{flag} names primitive {index}, but the mesh holds {}",
                match count {
                    1 => "primitive 0 alone".to_owned(),
                    _ => format!("primitives 0 to {}", count - 1),
                }
            ))
        })
    }

    /// The table by id.
    pub(crate) fn finish(self) -> IdVec<BMeshPrimitive, PrimitiveRecord> {
        IdVec::from_vec(self.records)
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::PrimitiveTable;
    use branded_id::U32Id;
    use voxsmith::operations::mesh::PrimitiveRecord;

    /// A declared primitive with no material selecting `select`.
    fn declared(select: &str) -> PrimitiveRecord {
        PrimitiveRecord {
            material_id: None,
            select: select.to_owned(),
            name: None,
            normal: true,
            uv_streams: None,
            attributes: Vec::new(),
        }
    }

    #[test]
    fn the_implicit_primitive_draws_material_0_only_when_materials_exist() {
        let with = PrimitiveTable::new(Vec::new(), 2).finish();
        assert_eq!(with.as_slice()[0].material_id, Some(U32Id::from_u32(0)));
        assert_eq!(with.as_slice()[0].select, "true");

        let without = PrimitiveTable::new(Vec::new(), 0).finish();
        assert_eq!(without.as_slice()[0].material_id, None);
    }

    #[test]
    fn declarations_replace_the_implicit_primitive_in_order() {
        let table = PrimitiveTable::new(vec![declared("a"), declared("b")], 0).finish();
        assert_eq!(table.len(), 2);
        assert_eq!(table.as_slice()[1].select, "b");
    }

    #[test]
    fn an_index_at_the_count_errors() {
        let mut table = PrimitiveTable::new(vec![declared("a")], 0);
        assert!(table.primitive("--primitive-name", 0).is_ok());
        assert!(table.primitive("--primitive-name", 1).is_err());
    }
}
