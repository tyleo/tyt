use crate::CliValue;
use voxsmith::VectorComponent;

impl CliValue for VectorComponent {
    const VARIANTS: &'static [Self] = &[
        VectorComponent::R,
        VectorComponent::G,
        VectorComponent::B,
        VectorComponent::A,
        VectorComponent::X,
        VectorComponent::Y,
        VectorComponent::Z,
        VectorComponent::W,
    ];

    fn name(self) -> &'static str {
        match self {
            VectorComponent::R => "r",
            VectorComponent::G => "g",
            VectorComponent::B => "b",
            VectorComponent::A => "a",
            VectorComponent::X => "x",
            VectorComponent::Y => "y",
            VectorComponent::Z => "z",
            VectorComponent::W => "w",
        }
    }

    fn help(self) -> &'static str {
        match self {
            VectorComponent::R => "Index 0, spelled `r`",
            VectorComponent::G => "Index 1, spelled `g`",
            VectorComponent::B => "Index 2, spelled `b`",
            VectorComponent::A => "Index 3, spelled `a`",
            VectorComponent::X => "Index 0, spelled `x`",
            VectorComponent::Y => "Index 1, spelled `y`",
            VectorComponent::Z => "Index 2, spelled `z`",
            VectorComponent::W => "Index 3, spelled `w`",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::CliValue;
    use voxsmith::VectorComponent;

    #[test]
    fn parses_the_color_aliases() {
        assert_eq!(VectorComponent::parse("r").unwrap(), VectorComponent::R);
        assert_eq!(VectorComponent::parse("g").unwrap(), VectorComponent::G);
        assert_eq!(VectorComponent::parse("b").unwrap(), VectorComponent::B);
        assert_eq!(VectorComponent::parse("a").unwrap(), VectorComponent::A);
    }

    #[test]
    fn parses_the_vector_aliases() {
        assert_eq!(VectorComponent::parse("x").unwrap(), VectorComponent::X);
        assert_eq!(VectorComponent::parse("y").unwrap(), VectorComponent::Y);
        assert_eq!(VectorComponent::parse("z").unwrap(), VectorComponent::Z);
        assert_eq!(VectorComponent::parse("w").unwrap(), VectorComponent::W);
    }

    #[test]
    fn rejects_unknown_components() {
        assert!(VectorComponent::parse("q").is_err());
        assert!(VectorComponent::parse("R").is_err());
        assert!(VectorComponent::parse("").is_err());
    }
}
