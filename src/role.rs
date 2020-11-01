use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    channel::permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
    guild::Permissions,
};

/// Error type for all calculator errors.
///
/// This will only return if [`Calculator::continue_on_missing_items`] wasn't
/// enabled.
///
/// [`Calculator::continue_on_missing_items`]: struct.Calculator.html#method.continue_on_missing_items
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RoleCalculatorError {
    /// Received Permission Overwrite is not a Role overwrite.
    PermissionOverwriteNotRole,
}

impl Display for RoleCalculatorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::PermissionOverwriteNotRole => {
                f.write_fmt(format_args!("permission overwrite is not role overwrite",))
            }
        }
    }
}

impl Error for RoleCalculatorError {}

/// Calculate the permissions of a role.
///
/// Created via the [`Calculator::role`] method.
///
/// Using the role calculator, you can calculate the role's permissions in a
/// given channel via [`in_channel`].
///
/// [`Calculator::role`]: struct.Calculator.html#method.role
/// [`in_channel`]: #method.in_channel
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use = "the role calculator isn't useful if you don't calculate permissions"]
pub struct RoleCalculator {
    permissions: Permissions,
}

impl RoleCalculator {
    /// Calculate the permissions of the role in the given channel.
    pub fn in_channel(
        mut self,
        permission_overwrite: PermissionOverwrite,
    ) -> Result<Permissions, RoleCalculatorError> {
        if !matches!(permission_overwrite.kind, PermissionOverwriteType::Role(_)) {
            return Err(RoleCalculatorError::PermissionOverwriteNotRole);
        }

        self.permissions.remove(permission_overwrite.deny);
        self.permissions.insert(permission_overwrite.allow);

        Ok(self.permissions)
    }
}

#[cfg(test)]
mod tests {
    use super::{RoleCalculator, RoleCalculatorError};
    use static_assertions::assert_impl_all;
    use std::{
        error::Error,
        fmt::{Debug, Display},
    };

    assert_impl_all!(
        RoleCalculatorError: Clone,
        Debug,
        Display,
        Error,
        Eq,
        PartialEq,
        Send,
        Sync
    );
    assert_impl_all!(RoleCalculator: Clone, Debug, Eq, PartialEq, Send, Sync);
}
