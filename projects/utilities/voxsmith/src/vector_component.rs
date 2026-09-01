/// One component of a vector value, addressed by index through either alias
/// set: the color letters `r`/`g`/`b`/`a` or the vector letters
/// `x`/`y`/`z`/`w`. The spelling survives for display.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

    /// The lowercase letter this component is spelled with.
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

#[cfg(test)]
mod tests {
    use crate::VectorComponent;

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
    fn the_letter_keeps_the_spelling() {
        assert_eq!(VectorComponent::A.letter(), 'a');
        assert_eq!(VectorComponent::W.letter(), 'w');
    }
}
