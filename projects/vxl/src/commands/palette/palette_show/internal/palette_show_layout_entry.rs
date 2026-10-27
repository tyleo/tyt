use crate::{CliValue, NamedCliValue, Width};
use serde::{
    Deserialize, Deserializer,
    de::{Error as DeError, MapAccess, Visitor, value::MapAccessDeserializer},
};
use std::{
    fmt::{Formatter, Result as FmtResult},
    num::NonZeroU8,
};
use voxsmith::operations::palette::{PaletteShowLabel, PaletteShowLayout, PaletteShowTableShape};

/// A layout with the display elements it takes. A profile writes the layout's
/// name, or an object whose `kind` selects the layout and whose other keys hold
/// that layout's elements.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaletteShowLayoutEntry {
    pub(crate) layout: PaletteShowLayout,

    pub(crate) label: Option<PaletteShowLabel>,

    pub(crate) header_level: Option<NonZeroU8>,

    pub(crate) table_shape: Option<PaletteShowTableShape>,

    pub(crate) width: Option<Width>,
}

impl From<PaletteShowLayout> for PaletteShowLayoutEntry {
    fn from(layout: PaletteShowLayout) -> Self {
        PaletteShowLayoutEntry {
            layout,
            label: None,
            header_level: None,
            table_shape: None,
            width: None,
        }
    }
}

impl<'de> Deserialize<'de> for PaletteShowLayoutEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(LayoutVisitor)
    }
}

struct LayoutVisitor;

impl<'de> Visitor<'de> for LayoutVisitor {
    type Value = PaletteShowLayoutEntry;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str("a layout name or an object naming the layout by `kind`")
    }

    fn visit_str<E: DeError>(self, name: &str) -> Result<Self::Value, E> {
        PaletteShowLayout::parse(name)
            .map(PaletteShowLayoutEntry::from)
            .map_err(E::custom)
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        LayoutObject::deserialize(MapAccessDeserializer::new(map)).map(PaletteShowLayoutEntry::from)
    }
}

/// A profile's `layout` object.
#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum LayoutObject {
    BoxHierarchy {},

    BoxTables {
        table_shape: Option<NamedCliValue<PaletteShowTableShape>>,
    },

    JsonCompact {},

    JsonPretty {},

    MdTables {
        table_shape: Option<NamedCliValue<PaletteShowTableShape>>,

        label: Option<NamedCliValue<PaletteShowLabel>>,

        header_level: Option<NonZeroU8>,
    },

    TextColumns {
        label: Option<NamedCliValue<PaletteShowLabel>>,

        header_level: Option<NonZeroU8>,
    },

    TextRows {
        label: Option<NamedCliValue<PaletteShowLabel>>,

        header_level: Option<NonZeroU8>,

        width: Option<Width>,
    },
}

impl From<LayoutObject> for PaletteShowLayoutEntry {
    fn from(object: LayoutObject) -> Self {
        let bare = PaletteShowLayoutEntry::from;

        match object {
            LayoutObject::BoxHierarchy {} => bare(PaletteShowLayout::BoxHierarchy),

            LayoutObject::BoxTables { table_shape } => PaletteShowLayoutEntry {
                table_shape: table_shape.map(|shape| shape.0),
                ..bare(PaletteShowLayout::BoxTables)
            },

            LayoutObject::JsonCompact {} => bare(PaletteShowLayout::JsonCompact),

            LayoutObject::JsonPretty {} => bare(PaletteShowLayout::JsonPretty),

            LayoutObject::MdTables {
                table_shape,
                label,
                header_level,
            } => PaletteShowLayoutEntry {
                table_shape: table_shape.map(|shape| shape.0),
                label: label.map(|label| label.0),
                header_level,
                ..bare(PaletteShowLayout::MdTables)
            },

            LayoutObject::TextColumns {
                label,
                header_level,
            } => PaletteShowLayoutEntry {
                label: label.map(|label| label.0),
                header_level,
                ..bare(PaletteShowLayout::TextColumns)
            },

            LayoutObject::TextRows {
                label,
                header_level,
                width,
            } => PaletteShowLayoutEntry {
                label: label.map(|label| label.0),
                header_level,
                width,
                ..bare(PaletteShowLayout::TextRows)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Width, commands::PaletteShowLayoutEntry};
    use std::num::NonZeroU8;
    use voxsmith::operations::palette::{
        PaletteShowLabel, PaletteShowLayout, PaletteShowTableShape,
    };

    /// The entry `json` reads into, or its error.
    fn read(json: &str) -> Result<PaletteShowLayoutEntry, String> {
        serde_json::from_str(json).map_err(|error| error.to_string())
    }

    #[test]
    fn a_bare_name_takes_no_elements() {
        assert_eq!(
            read(r#""json-pretty""#).unwrap(),
            PaletteShowLayoutEntry::from(PaletteShowLayout::JsonPretty)
        );
        assert!(read(r#""rows""#).is_err());
    }

    #[test]
    fn an_object_carries_its_layouts_elements() {
        assert_eq!(
            read(r#"{ "kind": "md-tables", "tableShape": "flat", "label": "header", "headerLevel": 2 }"#)
                .unwrap(),
            PaletteShowLayoutEntry {
                layout: PaletteShowLayout::MdTables,
                label: Some(PaletteShowLabel::Header),
                header_level: NonZeroU8::new(2),
                table_shape: Some(PaletteShowTableShape::Flat),
                width: None,
            }
        );
        assert_eq!(
            read(r#"{ "kind": "text-rows", "width": 80 }"#).unwrap(),
            PaletteShowLayoutEntry {
                width: Some(Width::Columns(80)),
                ..PaletteShowLayoutEntry::from(PaletteShowLayout::TextRows)
            }
        );
        assert_eq!(
            read(r#"{ "kind": "box-hierarchy" }"#).unwrap(),
            PaletteShowLayoutEntry::from(PaletteShowLayout::BoxHierarchy)
        );
    }

    #[test]
    fn an_element_foreign_to_the_layout_errors() {
        for json in [
            r#"{ "kind": "text-rows", "tableShape": "flat" }"#,
            r#"{ "kind": "md-tables", "width": 80 }"#,
            r#"{ "kind": "box-tables", "label": "concat" }"#,
            r#"{ "kind": "json-pretty", "label": "none" }"#,
            r#"{ "kind": "box-hierarchy", "width": 80 }"#,
        ] {
            let error = read(json).unwrap_err();
            assert!(error.contains("unknown field"), "{json}: {error}");
        }
    }

    #[test]
    fn a_bad_kind_or_value_errors() {
        assert!(read(r#"{ "tableShape": "flat" }"#).is_err());
        assert!(read(r#"{ "kind": "rows" }"#).is_err());
        assert!(read(r#"{ "kind": "md-tables", "tableShape": "wide" }"#).is_err());
        assert!(read(r#"{ "kind": "md-tables", "headerLevel": 0 }"#).is_err());
        assert!(read("3").is_err());
    }
}
