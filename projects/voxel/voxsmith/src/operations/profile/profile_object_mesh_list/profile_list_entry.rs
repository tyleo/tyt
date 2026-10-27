/// One profile of a
/// [`ProfileListGroup`](crate::operations::profile::ProfileListGroup).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileListEntry {
    /// The profile name.
    pub name: String,

    /// The profile's description, when it has one.
    pub description: Option<String>,
}
