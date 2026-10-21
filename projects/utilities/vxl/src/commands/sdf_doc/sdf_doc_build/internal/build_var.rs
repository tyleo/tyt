use serde_json::Value;
use std::str::FromStr;

/// A build variable from `--var <name>=<value>`.
#[derive(Clone, Debug, PartialEq)]
pub struct BuildVar {
    /// The name the model reads the variable by.
    pub name: String,

    /// The JSON value the model reads.
    pub value: Value,
}

impl FromStr for BuildVar {
    type Err = String;

    /// Parses `<name>=<value>`, splitting at the first `=` and reading the value
    /// as JSON.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let Some((name, value)) = text.split_once('=') else {
            return Err(format!("`{text}` is not `<name>=<value>`"));
        };
        if name.is_empty() {
            return Err(format!("`{text}` has no name before the `=`"));
        }

        let value = serde_json::from_str(value).map_err(|error| {
            format!(
                "the value of `{name}` is not JSON ({error}); a string takes quotes, as in \
                 `{name}='\"{value}\"'`"
            )
        })?;
        Ok(BuildVar {
            name: name.to_owned(),
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::BuildVar;
    use serde_json::json;

    #[test]
    fn the_value_after_the_first_equals_reads_as_json() {
        for (text, name, value) in [
            ("legs=4", "legs", json!(4)),
            ("size=[1, 2.5]", "size", json!([1, 2.5])),
            ("wood=\"oak\"", "wood", json!("oak")),
            ("rule=\"a=b\"", "rule", json!("a=b")),
        ] {
            let var: BuildVar = text.parse().unwrap();
            assert_eq!((var.name.as_str(), var.value), (name, value), "{text}");
        }
    }

    #[test]
    fn a_text_without_a_name_or_a_json_value_errors() {
        for (text, message) in [
            ("legs", "`legs` is not `<name>=<value>`"),
            ("=4", "`=4` has no name before the `=`"),
            (
                "wood=oak",
                "the value of `wood` is not JSON (expected value at line 1 column 1); a string \
                 takes quotes, as in `wood='\"oak\"'`",
            ),
        ] {
            assert_eq!(text.parse::<BuildVar>().unwrap_err(), message, "{text}");
        }
    }
}
