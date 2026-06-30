use crate::{
    AttributeType, ColorComponent, Dependencies, Error, Format, PaletteShowFormat, Result,
};
use clap::Parser;
use std::{
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

/// Prints one palette's selected attribute.
#[derive(Clone, Debug, Parser)]
#[command(name = "show")]
pub struct Show {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// Which palette to show, by index into the document's palettes.
    #[arg(value_name = "index", long, default_value_t = 0)]
    index: usize,

    /// Which attribute to show, a key optionally suffixed with a `.r`/`.g`/`.b`/
    /// `.a` color component, so `rgba` is the whole color and `rgba.a` one
    /// channel.
    #[arg(value_name = "attribute", long, default_value = "rgba")]
    attribute: String,

    /// How to interpret the attribute's values. Inferred from the stored value
    /// when omitted: a `#RRGGBBAA` string is a color and a number is a scalar.
    #[arg(value_name = "type", long = "type")]
    r#type: Option<AttributeType>,

    /// How to render each value.
    #[arg(value_name = "format", long, default_value = "auto")]
    format: PaletteShowFormat,

    /// Emit the selected attribute as JSON instead.
    #[arg(value_name = "json", long)]
    json: bool,
}

impl Show {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let (key, component) = parse_attribute(&self.attribute)
            .map_err(|message| Error::IO(IOError::new(ErrorKind::InvalidInput, message)))?;
        dependencies.palette_show(
            &self.input,
            self.from,
            self.index,
            &key,
            component,
            self.r#type,
            self.format,
            self.json,
        )
    }
}

/// Splits an `--attribute` value into its key and optional trailing color
/// component. A trailing single-letter `.r`/`.g`/`.b`/`.a` is the component; a
/// longer dotted suffix is part of the key, so dotted keys pass through whole,
/// the same split `--texture-map` applies. Unlike a `--texture-map` channel,
/// `--attribute` carries no `1-` inversion, constants, or `smoothness` alias.
fn parse_attribute(value: &str) -> std::result::Result<(String, Option<ColorComponent>), String> {
    let (key, component) = match value.rsplit_once('.') {
        Some((head, tail)) if tail.len() == 1 && tail.chars().all(|c| c.is_ascii_alphabetic()) => {
            (head, Some(tail.parse::<ColorComponent>()?))
        }
        _ => (value, None),
    };
    if key.is_empty() {
        return Err(format!("`{value}` names no attribute"));
    }
    Ok((key.to_string(), component))
}

#[cfg(test)]
mod tests {
    use crate::{ColorComponent, commands::show::parse_attribute};

    #[test]
    fn parses_a_bare_key() {
        assert_eq!(parse_attribute("rgba").unwrap(), ("rgba".to_string(), None));
        assert_eq!(
            parse_attribute("metallic").unwrap(),
            ("metallic".to_string(), None)
        );
    }

    #[test]
    fn parses_a_trailing_color_component() {
        assert_eq!(
            parse_attribute("rgba.a").unwrap(),
            ("rgba".to_string(), Some(ColorComponent::A))
        );
        assert_eq!(
            parse_attribute("rgba.r").unwrap(),
            ("rgba".to_string(), Some(ColorComponent::R))
        );
    }

    #[test]
    fn keeps_a_dotted_key_without_a_component() {
        assert_eq!(
            parse_attribute("my.attr").unwrap(),
            ("my.attr".to_string(), None)
        );
    }

    #[test]
    fn rejects_an_unknown_component() {
        assert!(parse_attribute("rgba.z").is_err());
    }

    #[test]
    fn rejects_an_empty_key() {
        assert!(parse_attribute("").is_err());
        assert!(parse_attribute(".a").is_err());
    }
}
