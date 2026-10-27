use serde::{Deserialize, Deserializer, de::Error as DeError};
use std::{fmt::Display, str::FromStr};

/// `none` or a value of `T`. Parses the literal `none` to [`NoneOr::None`],
/// else delegates to `T`'s own parse, so `T` is a newtype that validates the
/// inner value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoneOr<T> {
    /// The literal `none`.
    None,

    /// A parsed `T`.
    Value(T),
}

impl<T> NoneOr<T> {
    /// The value as an `Option`, `None` for the literal `none`.
    pub fn value(self) -> Option<T> {
        match self {
            NoneOr::None => None,
            NoneOr::Value(value) => Some(value),
        }
    }
}

impl<T> FromStr for NoneOr<T>
where
    T: FromStr,
    T::Err: Display,
{
    type Err = String;

    /// Parses `none` or delegates to `T`, stringifying its error.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("none") {
            return Ok(NoneOr::None);
        }

        value
            .parse::<T>()
            .map(NoneOr::Value)
            .map_err(|error| error.to_string())
    }
}

impl<'de, T> Deserialize<'de> for NoneOr<T>
where
    T: FromStr,
    T::Err: Display,
{
    /// Reads a string and parses it as the command line does.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::{NoneOr, Rgba};

    #[test]
    fn parses_none_case_insensitively() {
        assert_eq!("none".parse::<NoneOr<u8>>().unwrap(), NoneOr::None);
        assert_eq!("NONE".parse::<NoneOr<u8>>().unwrap(), NoneOr::None);
    }

    #[test]
    fn parses_an_inner_value() {
        assert_eq!("7".parse::<NoneOr<u8>>().unwrap(), NoneOr::Value(7));
    }

    #[test]
    fn propagates_an_inner_parse_error() {
        assert!("lots".parse::<NoneOr<u8>>().is_err());
    }

    #[test]
    fn value_maps_none_to_option() {
        assert_eq!(NoneOr::<u8>::None.value(), None);
        assert_eq!(NoneOr::Value(8u8).value(), Some(8));
    }

    #[test]
    fn deserializes_from_the_command_line_string() {
        assert_eq!(
            serde_json::from_str::<NoneOr<Rgba>>(r#""none""#).unwrap(),
            NoneOr::None
        );
        assert_eq!(
            serde_json::from_str::<NoneOr<Rgba>>(r##""#ff0000""##).unwrap(),
            NoneOr::Value(Rgba([255, 0, 0, 255]))
        );
        assert!(serde_json::from_str::<NoneOr<Rgba>>(r#""red""#).is_err());
    }
}
