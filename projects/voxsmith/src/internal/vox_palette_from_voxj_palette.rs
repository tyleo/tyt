use crate::vox_value_from_voxj_value;
use std::io;
use voxcore::VoxPalette;
use voxj::VoxjPalette;

/// Builds a [`VoxPalette`] from a [`VoxjPalette`], in listing order so each id
/// equals its voxj index. Duplicate attribute names are deduped last-wins (JSON
/// convention). Errors on a ragged cell row or a non-finite value.
pub(crate) fn vox_palette_from_voxj_palette(palette: &VoxjPalette) -> io::Result<VoxPalette> {
    let mut out = VoxPalette::default();

    // Dedup attribute names last-wins; remember each survivor's source column.
    let mut kept: Vec<(&String, usize)> = Vec::new();
    for (index, name) in palette.attributes.iter().enumerate() {
        if let Some(position) = kept.iter().position(|(kept_name, _)| *kept_name == name) {
            kept.remove(position);
        }
        kept.push((name, index));
    }
    for (name, _) in &kept {
        out.add_attribute((*name).clone());
    }

    for (index, row) in palette.data.iter().enumerate() {
        if row.len() != palette.attributes.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "palette cell {index} has {} values but the palette has {} attributes",
                    row.len(),
                    palette.attributes.len()
                ),
            ));
        }
        let values = kept
            .iter()
            .map(|&(_, source)| vox_value_from_voxj_value(&row[source]))
            .collect::<io::Result<Vec<_>>>()?;
        out.add_cell(values)
            .expect("one value per deduplicated attribute");
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::vox_palette_from_voxj_palette;
    use voxcore::VoxValue;
    use voxj::{VoxjPalette, VoxjValue};

    #[test]
    fn deduplicates_attribute_names_last_wins() {
        let palette = VoxjPalette {
            attributes: vec!["a".to_owned(), "b".to_owned(), "a".to_owned()],
            data: vec![vec![
                VoxjValue::Number(0.0),
                VoxjValue::Number(1.0),
                VoxjValue::Number(2.0),
            ]],
        };
        let out = vox_palette_from_voxj_palette(&palette).unwrap();

        // "a" deduped to its last occurrence; surviving columns are [b, a].
        let attributes: Vec<_> = out.iter_attributes().collect();
        assert_eq!(
            attributes.iter().map(|(_, name)| *name).collect::<Vec<_>>(),
            ["b", "a"]
        );

        let cell = out.iter_cells().next().unwrap();
        assert_eq!(
            out.cell_value(cell, attributes[0].0),
            Some(&VoxValue::Number(1.0)) // b -> source index 1
        );
        assert_eq!(
            out.cell_value(cell, attributes[1].0),
            Some(&VoxValue::Number(2.0)) // a -> source index 2 (last wins)
        );
    }

    #[test]
    fn rejects_ragged_cell_rows() {
        let palette = VoxjPalette {
            attributes: vec!["a".to_owned(), "b".to_owned()],
            data: vec![vec![VoxjValue::Number(0.0)]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }
}
