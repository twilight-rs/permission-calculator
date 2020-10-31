use crate::error::Error;
use std::collections::HashMap;
use twilight_model::{
    channel::{
        permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
        ChannelType,
    },
    guild::Permissions,
    id::{GuildId, RoleId, UserId},
};

/// Permissions associated with sending messages in a guild text channel.
const PERMISSIONS_MESSAGING: Permissions = Permissions::from_bits_truncate(
    Permissions::ATTACH_FILES.bits()
        | Permissions::EMBED_LINKS.bits()
        | Permissions::MENTION_EVERYONE.bits()
        | Permissions::SEND_TTS_MESSAGES.bits(),
);

/// Permissions associated with a guild only at the root level (i.e. not channel
/// related).
const PERMISSIONS_ROOT: Permissions = Permissions::from_bits_truncate(
    Permissions::ADMINISTRATOR.bits()
        | Permissions::BAN_MEMBERS.bits()
        | Permissions::CHANGE_NICKNAME.bits()
        | Permissions::KICK_MEMBERS.bits()
        | Permissions::MANAGE_EMOJIS.bits()
        | Permissions::MANAGE_GUILD.bits()
        | Permissions::MANAGE_NICKNAMES.bits()
        | Permissions::VIEW_AUDIT_LOG.bits()
        | Permissions::VIEW_GUILD_INSIGHTS.bits(),
);

/// Permissions associated with only guild text channels.
const PERMISSIONS_TEXT: Permissions = Permissions::from_bits_truncate(
    Permissions::ADD_REACTIONS.bits()
        | Permissions::ATTACH_FILES.bits()
        | Permissions::EMBED_LINKS.bits()
        | Permissions::MANAGE_MESSAGES.bits()
        | Permissions::MENTION_EVERYONE.bits()
        | Permissions::READ_MESSAGE_HISTORY.bits()
        | Permissions::SEND_MESSAGES.bits()
        | Permissions::SEND_TTS_MESSAGES.bits()
        | Permissions::USE_EXTERNAL_EMOJIS.bits(),
);

/// Permissions associated with only voice channels.
const PERMISSIONS_VOICE: Permissions = Permissions::from_bits_truncate(
    Permissions::CONNECT.bits()
        | Permissions::DEAFEN_MEMBERS.bits()
        | Permissions::MOVE_MEMBERS.bits()
        | Permissions::MUTE_MEMBERS.bits()
        | Permissions::PRIORITY_SPEAKER.bits()
        | Permissions::SPEAK.bits()
        | Permissions::STREAM.bits()
        | Permissions::USE_VAD.bits(),
);

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
    /// **Note** that this can be "dangerous", and can result in users "being
    /// able" to do things when in reality they can't.
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
        // If the user is the owner, then we can just return all of the
        // permissions.
        if self.user_id == self.guild_owner_id {
            return Ok(Permissions::all());
        }

        // The permissions that the @everyone role has is the baseline.
        let mut permissions = if let Some(permissions) = self.roles.get(&RoleId(self.guild_id.0)) {
            *permissions
        } else {
            #[cfg(feature = "tracing")]
            tracing::debug!(
                guild_id = %self.guild_id,
                "Everyone role not in guild",
            );

            // If the user wants to continue on missing items, then
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
                #[cfg(feature = "tracing")]
                tracing::debug!(
                    role_id = %role_id,
                    user_id = %self.user_id,
                    "User has role but it was not provided",
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

            permissions.insert(role_permissions);
        }

        Ok(permissions)
    }

    /// Calculate the permissions of the member in a channel, taking into
    /// account a combination of the guild-level permissions and channel-level
    /// permissions.
    ///
    /// When the "View Channel" permission is denied on the role level and isn't
    /// enabled on a role or the member or is denied on the member but isn't
    /// enabled on the member, then an empty permission set will be returned.
    ///
    /// When the "Send Messages" permission is denied and is not similarly
    /// enabled like above, then the "Attach Files", "Embed Links",
    /// "Mention Everyone", and "Send TTS Messages" permissions will not be
    /// present in the returned permission set.
    ///
    /// When the given channel type is not a guild text channel, then the
    /// following text permissions will not be present, even if enabled on the
    /// guild role level:
    ///
    /// - Add Reactions
    /// - Attach Files
    /// - Embed Links
    /// - Manage Messages
    /// - Mention Everyone
    /// - Read Message History
    /// - Send Messages
    /// - Send TTS Messages
    /// - Use External Emojis
    ///
    /// When the given channel type is not a guild voice channel, then the
    /// following voice permissions will not be present, even if enabled on the
    /// guild role level:
    ///
    /// - Deafen Members
    /// - Move Members
    /// - Mute Members
    /// - Priority Speaker
    /// - Speak
    /// - Stream
    /// - Use VAD
    ///
    /// The following guild level permissions will always be removed:
    ///
    /// - Administrator
    /// - Ban Members
    /// - Change Nickname
    /// - Kick Members
    /// - Manage Emojis
    /// - Manage Guild
    /// - Manage Nicknames
    /// - View Audit Log
    /// - View Guild Insights
    ///
    /// If you need to know a member's guild-level permissions (such as whether
    /// they have the "View Audit Log" permission), use [`permissions`].
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
    /// [`permissions`]: #method.permissions
    pub fn in_channel<U: IntoIterator<Item = &'a PermissionOverwrite> + Clone>(
        self,
        channel_type: ChannelType,
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

        // If the member or any of their roles denies the Send Messages
        // permission, then the rest of the messaging-related permissions can be
        // removed.
        let role_send_messages_denied = roles_deny.contains(Permissions::SEND_MESSAGES)
            && !roles_allow.contains(Permissions::SEND_MESSAGES)
            && !roles_allow.contains(Permissions::SEND_MESSAGES);

        let member_send_messages_denied = member_deny.contains(Permissions::SEND_MESSAGES)
            && !member_allow.contains(Permissions::SEND_MESSAGES);

        if member_send_messages_denied || role_send_messages_denied {
            member_allow.remove(PERMISSIONS_MESSAGING);
            roles_allow.remove(PERMISSIONS_MESSAGING);
            permissions.remove(PERMISSIONS_MESSAGING);
        }

        permissions.remove(roles_deny);
        permissions.insert(roles_allow);
        permissions.remove(member_deny);
        permissions.insert(member_allow);

        // Remove permissions that can't be used in a channel, i.e. are relevant
        // to guild-level permission calculating.
        permissions.remove(PERMISSIONS_ROOT);

        // Now remove permissions that can't be used in text or voice channels
        // based on this channel's type. This handles category channels by
        // removing all text and voice permissions.
        if channel_type != ChannelType::GuildText {
            permissions.remove(PERMISSIONS_TEXT);
        }

        if channel_type != ChannelType::GuildVoice {
            permissions.remove(PERMISSIONS_VOICE);
        }

        Ok(permissions)
    }
}

#[cfg(test)]
mod tests {
    use super::{Calculator, GuildId, MemberCalculator, RoleId, UserId};
    use static_assertions::{assert_impl_all, assert_obj_safe};
    use std::{collections::HashMap, fmt::Debug};
    use twilight_model::{
        channel::{
            permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
            ChannelType,
        },
        guild::Permissions,
    };

    assert_impl_all!(Calculator<'static>: Clone, Debug, Eq, PartialEq, Send, Sync);
    assert_impl_all!(MemberCalculator<'static, &[RoleId]>: Clone, Debug, Eq, PartialEq, Send, Sync);
    assert_obj_safe!(Calculator<'_>, MemberCalculator<'_, &[RoleId]>);

    // Test that a permission overwrite denying the "View Channel" permission
    // implicitly denies all other permissions.
    #[test]
    fn test_view_channel_deny_implicit() {
        let guild_id = GuildId(1);
        let guild_owner_id = UserId(2);
        let user_id = UserId(3);
        let member_roles = &[RoleId(4)];
        let mut roles = HashMap::with_capacity(1);
        roles.insert(
            RoleId(1),
            Permissions::SEND_MESSAGES | Permissions::MENTION_EVERYONE,
        );
        roles.insert(RoleId(4), Permissions::empty());

        // First, test when it's denied for an overwrite on a role the user has.
        let overwrites = &[PermissionOverwrite {
            allow: Permissions::SEND_TTS_MESSAGES,
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Role(RoleId(4)),
        }];

        let calculated = Calculator::new(guild_id, guild_owner_id, &roles)
            .member(user_id, member_roles)
            .in_channel(ChannelType::GuildText, overwrites)
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
            .in_channel(ChannelType::GuildText, overwrites)
            .unwrap();

        assert_eq!(calculated, Permissions::empty());
    }

    #[test]
    fn test_remove_text_perms_when_voice() {
        let guild_id = GuildId(1);
        let guild_owner_id = UserId(2);
        let user_id = UserId(3);
        let member_roles = &[RoleId(4)];
        let mut roles = HashMap::with_capacity(1);
        roles.insert(RoleId(1), Permissions::CONNECT);
        roles.insert(RoleId(4), Permissions::SEND_MESSAGES);

        let calculated = Calculator::new(guild_id, guild_owner_id, &roles)
            .member(user_id, member_roles)
            .in_channel(ChannelType::GuildVoice, &[])
            .unwrap();

        assert_eq!(calculated, Permissions::CONNECT);
    }

    #[test]
    fn test_remove_voice_perms_when_text() {
        let guild_id = GuildId(1);
        let guild_owner_id = UserId(2);
        let user_id = UserId(3);
        let member_roles = &[RoleId(4)];
        let mut roles = HashMap::with_capacity(1);
        roles.insert(RoleId(1), Permissions::CONNECT);
        roles.insert(RoleId(4), Permissions::SEND_MESSAGES);

        let calculated = Calculator::new(guild_id, guild_owner_id, &roles)
            .member(user_id, member_roles)
            .in_channel(ChannelType::GuildText, &[])
            .unwrap();

        assert_eq!(calculated, Permissions::SEND_MESSAGES);
    }

    // Test that denying the "Send Messages" permission denies all message
    // send related permissions.
    #[test]
    fn test_deny_send_messages_removes_related() {
        let guild_id = GuildId(1);
        let guild_owner_id = UserId(2);
        let user_id = UserId(3);
        let member_roles = &[RoleId(4)];
        let mut roles = HashMap::with_capacity(1);
        roles.insert(
            RoleId(1),
            Permissions::MANAGE_MESSAGES | Permissions::EMBED_LINKS | Permissions::MENTION_EVERYONE,
        );
        roles.insert(RoleId(4), Permissions::empty());

        // First, test when it's denied for an overwrite on a role the user has.
        let overwrites = &[PermissionOverwrite {
            allow: Permissions::ATTACH_FILES,
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(RoleId(4)),
        }];

        let calculated = Calculator::new(guild_id, guild_owner_id, &roles)
            .member(user_id, member_roles)
            .in_channel(ChannelType::GuildText, overwrites)
            .unwrap();

        assert_eq!(calculated, Permissions::MANAGE_MESSAGES);
    }
}
