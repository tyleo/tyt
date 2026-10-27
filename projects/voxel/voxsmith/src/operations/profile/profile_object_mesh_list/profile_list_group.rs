use crate::operations::profile::ProfileListEntry;

/// The profiles one origin supplies, an entry of a
/// [`profile_list`](crate::operations::profile::profile_list()).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileListGroup {
    /// The origin supplying the profiles, a config path or `built in`.
    pub origin: String,

    /// The profiles, in name order.
    pub profiles: Vec<ProfileListEntry>,
}
