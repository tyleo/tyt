/// The mode a rounding conversion snaps by.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rounding {
    Ceil,

    Floor,

    Round,
}
