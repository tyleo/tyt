use crate::{Error, Result};
use serde_json::{Map, Value};

/// The key under an object's `extras` this crate owns.
const VXL_KEY: &str = "vxl";

/// The key under [`VXL_KEY`] holding the named values.
const VALUES_KEY: &str = "values";

/// The key under [`VXL_KEY`] holding a primitive's name.
const NAME_KEY: &str = "name";

/// What this crate reads from and writes to an object's `extras` under the
/// `vxl` key. The rest of the `extras` stays in the ext.
#[derive(Debug, Default, PartialEq)]
pub struct VxlExtras {
    /// The `vxl.values` entries, by name.
    pub values: Map<String, Value>,

    /// The `vxl.name`, when present.
    pub name: Option<String>,
}

impl VxlExtras {
    /// Splits `extras` into the `vxl` entries this crate reads and the rest.
    /// Errors when `vxl`, `vxl.values`, or `vxl.name` has the wrong shape.
    pub fn split(extras: &Option<Value>) -> Result<(Option<Value>, Self)> {
        let Some(Value::Object(object)) = extras else {
            return Ok((extras.clone(), Self::default()));
        };

        let Some(vxl) = object.get(VXL_KEY) else {
            return Ok((extras.clone(), Self::default()));
        };

        let Value::Object(vxl) = vxl else {
            return Err(Error::invalid("extras `vxl` is not a JSON object"));
        };

        let mut rest = object.clone();
        let mut vxl = vxl.clone();

        let values = match vxl.remove(VALUES_KEY) {
            None => Map::new(),
            Some(Value::Object(values)) => values,
            Some(_) => return Err(Error::invalid("extras `vxl.values` is not a JSON object")),
        };

        let name = match vxl.remove(NAME_KEY) {
            None => None,
            Some(Value::String(name)) => Some(name),
            Some(_) => return Err(Error::invalid("extras `vxl.name` is not a string")),
        };

        if vxl.is_empty() {
            rest.remove(VXL_KEY);
        } else {
            rest.insert(VXL_KEY.to_owned(), Value::Object(vxl));
        }

        let rest = (!rest.is_empty()).then_some(Value::Object(rest));

        Ok((rest, Self { values, name }))
    }

    /// `rest` with these entries merged back under `vxl`. Errors when there
    /// is something to merge and `rest` or its `vxl` is not a JSON object.
    pub fn merge_into(self, rest: &Option<Value>) -> Result<Option<Value>> {
        if self.values.is_empty() && self.name.is_none() {
            return Ok(rest.clone());
        }

        let mut object = match rest {
            None => Map::new(),
            Some(Value::Object(object)) => object.clone(),
            Some(_) => {
                return Err(Error::invalid(
                    "the ext extras are not a JSON object to hold the `vxl` entries",
                ));
            }
        };

        let mut vxl = match object.remove(VXL_KEY) {
            None => Map::new(),
            Some(Value::Object(vxl)) => vxl,
            Some(_) => return Err(Error::invalid("the ext extras `vxl` is not a JSON object")),
        };

        if !self.values.is_empty() {
            vxl.insert(VALUES_KEY.to_owned(), Value::Object(self.values));
        }

        if let Some(name) = self.name {
            vxl.insert(NAME_KEY.to_owned(), Value::String(name));
        }

        object.insert(VXL_KEY.to_owned(), Value::Object(vxl));

        Ok(Some(Value::Object(object)))
    }
}

#[cfg(test)]
mod tests {
    use crate::VxlExtras;
    use serde_json::json;

    #[test]
    fn splits_and_merges_around_the_rest() {
        let extras = Some(json!({
            "other": 1,
            "vxl": { "values": { "a": [1, 2] }, "name": "shell", "keep": true }
        }));

        let (rest, vxl) = VxlExtras::split(&extras).unwrap();
        assert_eq!(rest, Some(json!({ "other": 1, "vxl": { "keep": true } })));
        assert_eq!(
            vxl.values,
            json!({ "a": [1, 2] }).as_object().unwrap().clone()
        );
        assert_eq!(vxl.name.as_deref(), Some("shell"));

        assert_eq!(vxl.merge_into(&rest).unwrap(), extras);
    }

    #[test]
    fn an_emptied_vxl_drops_out_and_nothing_merges_into_nothing() {
        let extras = Some(json!({ "vxl": { "values": {} } }));
        let (rest, vxl) = VxlExtras::split(&extras).unwrap();
        assert_eq!(rest, None);
        assert_eq!(vxl, VxlExtras::default());
        assert_eq!(VxlExtras::default().merge_into(&None).unwrap(), None);
    }

    #[test]
    fn wrong_shapes_error() {
        assert!(VxlExtras::split(&Some(json!({ "vxl": 3 }))).is_err());
        assert!(VxlExtras::split(&Some(json!({ "vxl": { "values": [] } }))).is_err());
        assert!(VxlExtras::split(&Some(json!({ "vxl": { "name": 3 } }))).is_err());
        assert!(
            VxlExtras {
                name: Some("x".to_owned()),
                ..Default::default()
            }
            .merge_into(&Some(json!(3)))
            .is_err()
        );
    }
}
