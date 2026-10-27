use serde::Deserialize;

/// A profile's description, held to one line of text because the listings
/// print it beside the profile name.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(try_from = "String")]
pub struct ProfileDescription(String);

impl ProfileDescription {
    /// The description text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ProfileDescription {
    type Error = String;

    /// Accepts one line holding more than whitespace.
    fn try_from(text: String) -> Result<Self, Self::Error> {
        if text.trim().is_empty() {
            return Err("a profile description must hold text".to_owned());
        }

        if text.contains(['\n', '\r']) {
            return Err(format!("the profile description {text:?} must be one line"));
        }

        Ok(ProfileDescription(text))
    }
}

#[cfg(test)]
mod tests {
    use crate::ProfileDescription;

    #[test]
    fn deserializes_one_line_of_text() {
        let description: ProfileDescription =
            serde_json::from_str(r#""A base color texture""#).unwrap();

        assert_eq!(description.as_str(), "A base color texture");
    }

    #[test]
    fn rejects_an_empty_or_blank_description() {
        assert!(serde_json::from_str::<ProfileDescription>(r#""""#).is_err());
        assert!(serde_json::from_str::<ProfileDescription>(r#""  ""#).is_err());
    }

    #[test]
    fn rejects_a_line_break() {
        let error = serde_json::from_str::<ProfileDescription>(r#""one\ntwo""#)
            .unwrap_err()
            .to_string();

        assert!(error.contains("must be one line"), "{error}");
        assert!(serde_json::from_str::<ProfileDescription>(r#""one\rtwo""#).is_err());
    }
}
