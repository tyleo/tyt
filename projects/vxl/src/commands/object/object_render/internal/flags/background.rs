use crate::commands::SrgbColor;
use serde::Deserialize;
use std::str::FromStr;
use ty_math::TySrgbU8;

/// What fills the pixels no ray hits.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(try_from = "String")]
pub enum Background {
    /// The literal `transparent`.
    Transparent,

    /// A `#RRGGBB` color.
    Color(SrgbColor),
}

impl Background {
    /// The color under the misses, or `None` to leave them transparent.
    pub(crate) fn color(self) -> Option<TySrgbU8> {
        match self {
            Background::Transparent => None,
            Background::Color(color) => Some(color.0),
        }
    }
}

impl FromStr for Background {
    type Err = String;

    /// Parses `transparent` or a `#RRGGBB` hex.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == "transparent" {
            return Ok(Background::Transparent);
        }

        value
            .parse()
            .map(Background::Color)
            .map_err(|_| format!("`{value}` is not a background; use transparent or a #RRGGBB hex"))
    }
}

impl TryFrom<String> for Background {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::{Background, SrgbColor};
    use ty_math::TySrgbU8;

    #[test]
    fn transparent_or_a_hex() {
        assert_eq!(
            "transparent".parse::<Background>(),
            Ok(Background::Transparent)
        );
        assert_eq!(
            "#102030".parse::<Background>(),
            Ok(Background::Color(SrgbColor(TySrgbU8::new(16, 32, 48))))
        );
        assert!("clear".parse::<Background>().is_err());
        assert_eq!(
            serde_json::from_str::<Background>(r#""transparent""#).unwrap(),
            Background::Transparent
        );
    }
}
