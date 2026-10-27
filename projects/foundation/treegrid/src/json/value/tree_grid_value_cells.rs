use crate::{TreeGridJsonCells, TreeGridValue, TreeGridValueCells};
use serde_json::Value;

impl TreeGridJsonCells for TreeGridValueCells {
    fn json(&self, value: &TreeGridValue) -> Value {
        Value::String(value.text.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::{TreeGridJsonCells, TreeGridValue, TreeGridValueCells};
    use serde_json::Value;

    #[test]
    fn json_is_always_a_string_of_the_text() {
        let value = TreeGridValue::new("{node: 0}");

        assert_eq!(
            TreeGridValueCells.json(&value),
            Value::String("{node: 0}".to_owned())
        );
    }
}
