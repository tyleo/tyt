use std::str::FromStr;

/// One component of a vector value, addressed by index through either
/// alias set: the color letters `r`/`g`/`b`/`a` or the vector letters
/// `x`/`y`/`z`/`w`. Both aliases of an index address the same
/// component; the parsed spelling survives for display.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VectorComponent {
    /// Index 0, spelled `r`.
    R,
    /// Index 1, spelled `g`.
    G,
    /// Index 2, spelled `b`.
    B,
    /// Index 3, spelled `a`.
    A,
    /// Index 0, spelled `x`.
    X,
    /// Index 1, spelled `y`.
    Y,
    /// Index 2, spelled `z`.
    Z,
    /// Index 3, spelled `w`.
    W,
}

impl VectorComponent {
    /// The `0..3` index this component addresses.
    pub fn index(self) -> usize {
        match self {
            VectorComponent::R | VectorComponent::X => 0,
            VectorComponent::G | VectorComponent::Y => 1,
            VectorComponent::B | VectorComponent::Z => 2,
            VectorComponent::A | VectorComponent::W => 3,
        }
    }

    /// The lowercase letter this component was spelled with.
    pub fn letter(self) -> char {
        match self {
            VectorComponent::R => 'r',
            VectorComponent::G => 'g',
            VectorComponent::B => 'b',
            VectorComponent::A => 'a',
            VectorComponent::X => 'x',
            VectorComponent::Y => 'y',
            VectorComponent::Z => 'z',
            VectorComponent::W => 'w',
        }
    }
}

impl FromStr for VectorComponent {
    type Err = String;

    /// Parses a lowercase letter from either alias set.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "r" => Ok(VectorComponent::R),
            "g" => Ok(VectorComponent::G),
            "b" => Ok(VectorComponent::B),
            "a" => Ok(VectorComponent::A),
            "x" => Ok(VectorComponent::X),
            "y" => Ok(VectorComponent::Y),
            "z" => Ok(VectorComponent::Z),
            "w" => Ok(VectorComponent::W),
            _ => Err(format!(
                "`{value}` is not a vector component; use r, g, b, a, x, y, z, or w"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::VectorComponent;

    #[test]
    fn parses_the_color_aliases() {
        assert_eq!("r".parse::<VectorComponent>().unwrap(), VectorComponent::R);
        assert_eq!("g".parse::<VectorComponent>().unwrap(), VectorComponent::G);
        assert_eq!("b".parse::<VectorComponent>().unwrap(), VectorComponent::B);
        assert_eq!("a".parse::<VectorComponent>().unwrap(), VectorComponent::A);
    }

    #[test]
    fn parses_the_vector_aliases() {
        assert_eq!("x".parse::<VectorComponent>().unwrap(), VectorComponent::X);
        assert_eq!("y".parse::<VectorComponent>().unwrap(), VectorComponent::Y);
        assert_eq!("z".parse::<VectorComponent>().unwrap(), VectorComponent::Z);
        assert_eq!("w".parse::<VectorComponent>().unwrap(), VectorComponent::W);
    }

    #[test]
    fn both_alias_sets_share_the_indices() {
        assert_eq!(VectorComponent::R.index(), VectorComponent::X.index());
        assert_eq!(VectorComponent::G.index(), VectorComponent::Y.index());
        assert_eq!(VectorComponent::B.index(), VectorComponent::Z.index());
        assert_eq!(VectorComponent::A.index(), VectorComponent::W.index());
        assert_eq!(VectorComponent::R.index(), 0);
        assert_eq!(VectorComponent::W.index(), 3);
    }

    #[test]
    fn the_letter_keeps_the_parsed_spelling() {
        assert_eq!(VectorComponent::A.letter(), 'a');
        assert_eq!(VectorComponent::W.letter(), 'w');
    }

    #[test]
    fn rejects_unknown_components() {
        assert!("q".parse::<VectorComponent>().is_err());
        assert!("R".parse::<VectorComponent>().is_err());
        assert!("".parse::<VectorComponent>().is_err());
    }
}
