use crate::{
    Error, Result,
    operations::mesh::{MeshElement, Transfer, encode_components},
};
use serde_json::Value as JsonValue;
use vox_value_language::{Components, Scalar, Value};

/// The JSON text of `value` under `transfer`, indented for a key of a file's
/// object. An entry of two or more components writes as a row, and an array
/// writes one row per entry.
pub(crate) fn json_text(
    element: &MeshElement,
    value: &Value,
    transfer: Transfer,
) -> Result<String> {
    if transfer == Transfer::Srgb && value.scalar() != Scalar::F32 {
        return Err(Error::mesh_record(
            element.clone(),
            format!("is a {}, and `srgb` transfers f32 alone", value.to_type()),
        ));
    }

    let width = value.dimension().width();

    let entries: Vec<String> = match value.components() {
        Components::Bool(components) => rows(components, width, |flag| flag.to_string()),

        Components::F32(components) => components
            .chunks_exact(width)
            .map(|entry| {
                let encoded = encode_components(element, entry, transfer, false)?;

                Ok(row(encoded
                    .iter()
                    .map(|&component| {
                        serde_json::to_string(&(component as f32))
                            .expect("a finite float serializes")
                    })
                    .collect()))
            })
            .collect::<Result<_>>()?,

        Components::String(components) => rows(components, width, |text| {
            JsonValue::String(text.clone()).to_string()
        }),

        Components::U8(components) => rows(components, width, |number| {
            JsonValue::from(*number).to_string()
        }),

        Components::U16(components) => rows(components, width, |number| {
            JsonValue::from(*number).to_string()
        }),

        Components::U32(components) => rows(components, width, |number| {
            JsonValue::from(*number).to_string()
        }),
    };

    Ok(if value.domain().is_array() {
        if entries.is_empty() {
            "[]".to_owned()
        } else {
            format!("[\n    {}\n  ]", entries.join(",\n    "))
        }
    } else {
        entries
            .into_iter()
            .next()
            .expect("a plain value holds one entry")
    })
}

/// Each entry of `components` as a row of `width` leaves.
fn rows<T>(components: &[T], width: usize, leaf: impl Fn(&T) -> String) -> Vec<String> {
    components
        .chunks_exact(width)
        .map(|entry| row(entry.iter().map(&leaf).collect()))
        .collect()
}

/// One entry's text, bare for a lone leaf and bracketed for more.
fn row(leaves: Vec<String>) -> String {
    match leaves.as_slice() {
        [leaf] => leaf.clone(),
        _ => format!("[{}]", leaves.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::mesh::{MeshElement, Transfer, json_text};
    use vox_value_language::{Components, Dimension, Domain, Value};

    fn element() -> MeshElement {
        MeshElement::File {
            file: "bar.json".to_owned(),
        }
    }

    fn text(value: Value, transfer: Transfer) -> String {
        json_text(&element(), &value, transfer).unwrap()
    }

    #[test]
    fn a_plain_value_writes_bare_and_an_array_writes_rows() {
        let plain = Value::new(Domain::Plain, Dimension::Vec1, Components::F32(vec![0.4])).unwrap();
        assert_eq!(text(plain, Transfer::Linear), "0.4");

        let color = Value::new(
            Domain::Plain,
            Dimension::Vec4,
            Components::F32(vec![1.0, 0.0, 0.0, 1.0]),
        )
        .unwrap();
        assert_eq!(text(color, Transfer::Linear), "[1.0, 0.0, 0.0, 1.0]");

        let swatches = Value::new(
            Domain::Swatch,
            Dimension::Vec2,
            Components::U8(vec![1, 2, 3, 4]),
        )
        .unwrap();
        assert_eq!(
            text(swatches, Transfer::Linear),
            "[\n    [1, 2],\n    [3, 4]\n  ]"
        );

        let flags = Value::new(
            Domain::Face,
            Dimension::Vec1,
            Components::Bool(vec![true, false]),
        )
        .unwrap();
        assert_eq!(
            text(flags, Transfer::Linear),
            "[\n    true,\n    false\n  ]"
        );

        let names = Value::new(
            Domain::Plain,
            Dimension::Vec1,
            Components::String(vec!["a \"b\"".to_owned()]),
        )
        .unwrap();
        assert_eq!(text(names, Transfer::Linear), "\"a \\\"b\\\"\"");

        let none = Value::new(Domain::Face, Dimension::Vec1, Components::U32(vec![])).unwrap();
        assert_eq!(text(none, Transfer::Linear), "[]");
    }

    #[test]
    fn srgb_curves_the_floats_and_refuses_every_other_scalar() {
        let grey = Value::new(Domain::Plain, Dimension::Vec1, Components::F32(vec![0.5])).unwrap();
        let curved: f32 = text(grey, Transfer::Srgb).parse().unwrap();
        assert!((curved - 0.7354).abs() < 1e-4);

        let flag =
            Value::new(Domain::Plain, Dimension::Vec1, Components::Bool(vec![true])).unwrap();
        assert!(json_text(&element(), &flag, Transfer::Srgb).is_err());

        let count = Value::new(Domain::Plain, Dimension::Vec1, Components::U32(vec![1])).unwrap();
        assert!(json_text(&element(), &count, Transfer::Srgb).is_err());
    }
}
