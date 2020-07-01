use crate::{error::Error, role::Role};
use std::collections::HashMap;
use twilight_model::{
    channel::permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
    guild::Permissions,
    id::{GuildId, RoleId, UserId},
};

/// A calculator to calculate permissions of various things within in a guild.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use = "the calculator isn't useful if you don't calculate the permissions of something with it"]
pub struct Calculator<'a> {
    continue_on_missing_items: bool,
    id: GuildId,
    owner_id: UserId,
    roles: &'a HashMap<RoleId, Role>,
}

impl<'a> Calculator<'a> {
    /// Create a new permission calculator for a guild.
    ///
    /// Use the methods on this calculator to create new, more specific
    /// calculators.
    pub fn new(id: GuildId, owner_id: UserId, roles: &'a HashMap<RoleId, Role>) -> Self {
        Self {
            continue_on_missing_items: false,
            id,
            owner_id,
            roles,
        }
    }

    /// Whether to continue when items are missing from the cache, such as when
    /// a user has a certain role but that role doesn't exist.
    ///
    /// If this is `true`, then the calculated permissions may be incomplete or
    /// invalid. If this is `false`, then an error will return when an item is
    /// missing.
    ///
    /// The default is `false`.
    pub fn continue_on_missing_items(mut self, continue_on_missing_items: bool) -> Self {
        self.continue_on_missing_items = continue_on_missing_items;

        self
    }

    /// Create a calculator to calculate the permissions of a member.
    ///
    /// Using the returned member calculator, you can calculate the permissions
    /// of the member [across the guild][`permissions`] or
    /// [in a specified channel][`in_channel`].
    ///
    /// [`in_channel`]: struct.MemberCalculator.html#method.in_channel
    /// [`permissions`]: struct.MemberCalculator.html#method.permissions
    pub fn member<T: IntoIterator<Item = &'a RoleId> + Clone>(
        self,
        user_id: UserId,
        member_role_ids: T,
    ) -> MemberCalculator<'a, T> {
        MemberCalculator {
            continue_on_missing_items: self.continue_on_missing_items,
            guild_id: self.id,
            guild_owner_id: self.owner_id,
            member_role_ids,
            roles: self.roles,
            user_id,
        }
    }
}

/// Calculate the permissions of a member.
///
/// Created via the [`Calculator::member`] method.
///
/// Using the member calculator, you can calculate the member's permissions in
/// a given channel via [`in_channel`].
///
/// [`Calculator::member`]: struct.Calculator.html#method.member
/// [`in_channel`]: #method.in_channel
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use = "the member calculator isn't useful if you don't calculate permissions"]
pub struct MemberCalculator<'a, T: IntoIterator<Item = &'a RoleId> + Clone> {
    continue_on_missing_items: bool,
    guild_id: GuildId,
    guild_owner_id: UserId,
    member_role_ids: T,
    roles: &'a HashMap<RoleId, Role>,
    user_id: UserId,
}

impl<'a, T: IntoIterator<Item = &'a RoleId> + Clone> MemberCalculator<'a, T> {
    /// Calculate the guild-level permissions of a member.
    ///
    /// # Errors
    ///
    /// If [`Calculator::continue_on_missing_items`] wasn't enabled, then this
    /// returns [`Error::EveryoneRoleMissing`] if the `@everyone` role with the
    /// same ID as the guild wasn't found in the given guild roles map.
    ///
    /// [`Calculator::continue_on_missing_items`]: struct.Calculator.html#method.continue_on_missing_items
    /// [`Error::EveryoneRoleMissing`]: enum.Error.html#method.EveryoneRoleMissing
    pub fn permissions(&self) -> Result<Permissions, Error> {
        // The owner has all permissions.
        if self.user_id == self.guild_owner_id {
            return Ok(Permissions::all());
        }

        // The permissions that everyone has is the baseline.
        let mut permissions = if let Some(role) = self.roles.get(&RoleId(self.guild_id.0)) {
            role.permissions
        } else {
            #[cfg(feature = "log")]
            log::debug!("Everyone role not in guild {}", self.guild_id,);

            if self.continue_on_missing_items {
                Permissions::empty()
            } else {
                return Err(Error::EveryoneRoleMissing {
                    guild_id: self.guild_id,
                });
            }
        };

        // Permissions on a user's roles are simply additive.
        for role_id in self.member_role_ids.clone() {
            let role = if let Some(role) = self.roles.get(&role_id) {
                role
            } else {
                #[cfg(feature = "log")]
                log::debug!(
                    "User {} has role {} but it was not provided",
                    self.user_id,
                    role_id,
                );

                if self.continue_on_missing_items {
                    continue;
                } else {
                    return Err(Error::MemberRoleMissing {
                        role_id: *role_id,
                        user_id: self.user_id,
                    });
                }
            };

            if permissions.contains(Permissions::ADMINISTRATOR) {
                return Ok(Permissions::all());
            }

            permissions |= role.permissions;
        }

        Ok(permissions)
    }

    /// Calculate the permissions of the member in a channel, taking into
    /// account a combination of the guild-level permissions and channel-level
    /// permissions.
    ///
    /// # Examples
    ///
    /// See the crate-level documentation for an example.
    ///
    /// # Errors
    ///
    /// If [`Calculator::continue_on_missing_items`] wasn't enabled, then this
    /// returns [`Error::EveryoneRoleMissing`] if the `@everyone` role with the
    /// same ID as the guild wasn't found in the given guild roles map.
    ///
    /// If [`Calculator::continue_on_missing_items`] wasn't enabled, then this
    /// returns [`Error::MemberRoleMissing`] if one of the specified user's
    /// guild roles was missing.
    ///
    /// [`Calculator::continue_on_missing_items`]: struct.Calculator.html#method.continue_on_missing_items
    /// [`Error::EveryoneRoleMissing`]: enum.Error.html#method.EveryoneRoleMissing
    /// [`Error::MemberRoleMissing`]: enum.Error.html#method.MemberRoleMissing
    pub fn in_channel<U: IntoIterator<Item = &'a PermissionOverwrite> + Clone>(
        self,
        channel_overwrites: U,
    ) -> Result<Permissions, Error> {
        let mut permissions = self.permissions()?;

        let mut data = Vec::new();

        for overwrite in channel_overwrites.clone() {
            if let PermissionOverwriteType::Role(role) = overwrite.kind {
                if role.0 != self.guild_id.0
                    && !self.member_role_ids.clone().into_iter().any(|r| *r == role)
                {
                    continue;
                }

                if let Some(role) = self.roles.get(&role) {
                    data.push((role.position, overwrite.deny, overwrite.allow));
                }
            }
        }

        data.sort_by(|a, b| a.0.cmp(&b.0));

        for overwrite in data {
            permissions = (permissions & !overwrite.1) | overwrite.2;
        }

        for overwrite in channel_overwrites {
            if PermissionOverwriteType::Member(self.user_id) != overwrite.kind {
                continue;
            }

            permissions = (permissions & !overwrite.deny) | overwrite.allow;
        }

        Ok(permissions)
    }
}

#[cfg(test)]
mod tests {
    use super::{Calculator, MemberCalculator};
    use static_assertions::assert_impl_all;
    use std::fmt::Debug;

    assert_impl_all!(Calculator<'static>: Clone, Debug, Eq, PartialEq);
    assert_impl_all!(MemberCalculator<'static, &[_]>: Clone, Debug, Eq, PartialEq);
}
