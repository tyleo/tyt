use crate::ProfileDescription;

/// A profile a [`ProfileSet`](crate::ProfileSet) holds.
pub trait Profile {
    /// The description the profile listings print beside the profile name.
    fn description(&self) -> Option<&ProfileDescription>;
}
