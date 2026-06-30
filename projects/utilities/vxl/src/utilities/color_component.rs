use std::str::FromStr;

/// A color's red, green, blue, or alpha component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorComponent {
    /// The red component.
    R,
    /// The green component.
    G,
    /// The blue component.
    B,
    /// The straight-alpha component.
    A,
}

impl FromStr for ColorComponent {
    type Err = String;

    /// Parses a lowercase `r`, `g`, `b`, or `a`.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "r" => Ok(ColorComponent::R),
            "g" => Ok(ColorComponent::G),
            "b" => Ok(ColorComponent::B),
            "a" => Ok(ColorComponent::A),
            _ => Err(format!(
                "`{value}` is not a color component; use r, g, b, or a"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ColorComponent;

    #[test]
    fn parses_lowercase_components() {
        assert_eq!("r".parse::<ColorComponent>().unwrap(), ColorComponent::R);
        assert_eq!("g".parse::<ColorComponent>().unwrap(), ColorComponent::G);
        assert_eq!("b".parse::<ColorComponent>().unwrap(), ColorComponent::B);
        assert_eq!("a".parse::<ColorComponent>().unwrap(), ColorComponent::A);
    }

    #[test]
    fn rejects_unknown_components() {
        assert!("z".parse::<ColorComponent>().is_err());
        assert!("R".parse::<ColorComponent>().is_err());
        assert!("".parse::<ColorComponent>().is_err());
    }
}
