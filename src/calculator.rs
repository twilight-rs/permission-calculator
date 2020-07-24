use crate::error::Error;
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
    roles: &'a HashMap<RoleId, Permissions>,
}

impl<'a> Calculator<'a> {
    /// Create a new permission calculator for a guild.
    ///
    /// Use the methods on this calculator to create new, more specific
    /// calculators.
    pub fn new(id: GuildId, owner_id: UserId, roles: &'a HashMap<RoleId, Permissions>) -> Self {
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
pub struct MemberCalculator<'a, T> {
    continue_on_missing_items: bool,
    guild_id: GuildId,
    guild_owner_id: UserId,
    member_role_ids: T,
    roles: &'a HashMap<RoleId, Permissions>,
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
        let mut permissions = if let Some(permissions) = self.roles.get(&RoleId(self.guild_id.0)) {
            *permissions
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
            let role_permissions = if let Some(role) = self.roles.get(&role_id) {
                *role
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

            permissions |= role_permissions;
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
    #[allow(unused)]
    pub fn in_channel<U: IntoIterator<Item = &'a PermissionOverwrite> + Clone>(
        self,
        channel_overwrites: U,
    ) -> Result<Permissions, Error> {
        let mut permissions = self.permissions()?;

        // Hierarchy documentation:
        // <https://discord.com/developers/docs/topics/permissions#permission-overwrites>
        let mut member_allow = Permissions::empty();
        let mut member_deny = Permissions::empty();
        let mut roles_allow = Permissions::empty();
        let mut roles_deny = Permissions::empty();

        for overwrite in channel_overwrites.clone() {
            match overwrite.kind {
                PermissionOverwriteType::Role(role) => {
                    // We need to process the @everyone role first, so apply it
                    // straight to the permissions. The other roles' permissions
                    // will be applied later.
                    if role.0 == self.guild_id.0 {
                        permissions.remove(overwrite.deny);
                        permissions.insert(overwrite.allow);

                        continue;
                    }

                    if !self.member_role_ids.clone().into_iter().any(|r| *r == role) {
                        continue;
                    }

                    roles_allow.insert(overwrite.allow);
                    roles_deny.insert(overwrite.deny);
                }
                PermissionOverwriteType::Member(user_id) if user_id == self.user_id => {
                    member_allow.insert(overwrite.allow);
                    member_deny.insert(overwrite.deny);
                }
                PermissionOverwriteType::Member(_) => {}
            }
        }

        let role_view_channel_denied = roles_deny.contains(Permissions::VIEW_CHANNEL)
            && !roles_allow.contains(Permissions::VIEW_CHANNEL)
            && !roles_allow.contains(Permissions::VIEW_CHANNEL);

        let member_view_channel_denied = member_deny.contains(Permissions::VIEW_CHANNEL)
            && !member_allow.contains(Permissions::VIEW_CHANNEL);

        if member_view_channel_denied || role_view_channel_denied {
            return Ok(Permissions::empty());
        }

        permissions.remove(roles_deny);
        permissions.insert(roles_allow);
        permissions.remove(member_deny);
        permissions.insert(member_allow);

        Ok(permissions)
    }
}

#[cfg(test)]
mod tests {
    use super::{Calculator, GuildId, MemberCalculator, RoleId, UserId};
    use static_assertions::assert_impl_all;
    use std::{collections::HashMap, fmt::Debug};
    use twilight_model::{
        channel::permission_overwrite::{PermissionOverwriteType, PermissionOverwrite},
        guild::Permissions,
    };

    assert_impl_all!(Calculator<'static>: Clone, Debug, Eq, PartialEq);
    assert_impl_all!(MemberCalculator<'static, &[RoleId]>: Clone, Debug, Eq, PartialEq);

    // Test that a permission overwrite denying the "View Channel" permission
    // implicitly denies all other permissions.
    #[test]
    fn test_view_channel_deny_implicit() {
        let guild_id = GuildId(1);
        let guild_owner_id = UserId(2);
        let user_id = UserId(3);
        let member_roles = &[RoleId(4)];
        let mut roles = HashMap::with_capacity(1);
        roles.insert(RoleId(1), Permissions::SEND_MESSAGES | Permissions::MENTION_EVERYONE);
        roles.insert(RoleId(4), Permissions::empty());

        // First, test when it's denied for an overwrite on a role the user has.
        let overwrites = &[PermissionOverwrite {
            allow: Permissions::SEND_TTS_MESSAGES,
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Role(RoleId(4)),
        }];

        let calculated = Calculator::new(guild_id, guild_owner_id, &roles)
            .member(user_id, member_roles)
            .in_channel(overwrites)
            .unwrap();

        assert_eq!(calculated, Permissions::empty());

        // And now that it's denied for an overwrite on the member.
        let overwrites = &[PermissionOverwrite {
            allow: Permissions::SEND_TTS_MESSAGES,
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Member(UserId(3)),
        }];

        let calculated = Calculator::new(guild_id, guild_owner_id, &roles)
            .member(user_id, member_roles)
            .in_channel(overwrites)
            .unwrap();

        assert_eq!(calculated, Permissions::empty());
    }
}
