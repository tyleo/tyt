/// A palette attribute value. The recommended attributes are colors
/// (`#RRGGBBAA` strings) and numbers; booleans are accepted for custom keys.
#[derive(Clone, Debug, PartialEq)]
pub enum AttrValue {
    Number(f64),
    Text(String),
    Bool(bool),
}

impl From<f64> for AttrValue {
    fn from(v: f64) -> Self {
        AttrValue::Number(v)
    }
}

impl From<String> for AttrValue {
    fn from(v: String) -> Self {
        AttrValue::Text(v)
    }
}
