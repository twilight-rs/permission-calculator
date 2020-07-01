use twilight_model::guild::{Permissions, Role as TwilightRole};

/// Basic information about a guild role's guild-level granted permissions and
/// position in the role list.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use = "roles must be given to the calculator"]
#[non_exhaustive]
pub struct Role {
    /// The permissions for the role.
    pub permissions: Permissions,
    /// The position of the role.
    pub position: i64,
}

impl Role {
    /// Create a new role knowing the position and guild-level permissions.
    pub fn new(position: i64, permissions: Permissions) -> Self {
        Self::from((position, permissions))
    }
}

impl From<(i64, Permissions)> for Role {
    fn from((position, permissions): (i64, Permissions)) -> Self {
        Self {
            permissions,
            position,
        }
    }
}

impl From<TwilightRole> for Role {
    fn from(role: TwilightRole) -> Self {
        Self::from(&role)
    }
}

impl From<&'_ TwilightRole> for Role {
    fn from(role: &'_ TwilightRole) -> Self {
        Self {
            permissions: role.permissions,
            position: role.position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Role;
    use static_assertions::assert_impl_all;
    use std::fmt::Debug;

    assert_impl_all!(Role: Clone, Debug, Eq, PartialEq);
}
