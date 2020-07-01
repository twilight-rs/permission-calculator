use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::id::{GuildId, RoleId, UserId};

/// Error type for all calculator errors.
///
/// This will only return if [`Calculator::continue_on_missing_items`] wasn't
/// enabled.
///
/// [`Calculator::continue_on_missing_items`]: struct.Calculator.html#method.continue_on_missing_items
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// The guild's `@everyone` role was missing from the guild's role list.
    EveryoneRoleMissing {
        /// The ID of the guild and role.
        guild_id: GuildId,
    },
    /// One of the member's roles is missing from the guild's role list.
    MemberRoleMissing {
        /// The ID of the missing role that the member has.
        role_id: RoleId,
        /// The ID of the user.
        user_id: UserId,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::EveryoneRoleMissing { guild_id } => f.write_fmt(format_args!(
                "the @everyone role is missing for guild {}",
                guild_id
            )),
            Self::MemberRoleMissing { role_id, user_id } => f.write_fmt(format_args!(
                "member {} is missing role {}",
                user_id, role_id
            )),
        }
    }
}

impl StdError for Error {}

#[cfg(test)]
mod tests {
    use super::Error;
    use static_assertions::assert_impl_all;
    use std::{
        error::Error as StdError,
        fmt::{Debug, Display},
    };

    assert_impl_all!(Error: Clone, Debug, Display, Eq, PartialEq, StdError);
}
