use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
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

/// Error type for all calculator errors.
///
/// This will only return if [`Calculator::continue_on_missing_items`] wasn't
/// enabled.
///
/// [`Calculator::continue_on_missing_items`]: struct.Calculator.html#method.continue_on_missing_items
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MemberCalculatorError {
    /// `@everyone` role is missing from the guild's role list.
    EveryoneRoleMissing {
        /// ID of the guild and role.
        guild_id: GuildId,
    },
}

impl Display for MemberCalculatorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::EveryoneRoleMissing { guild_id } => f.write_fmt(format_args!(
                "@everyone role is missing for guild {}",
                guild_id
            )),
        }
    }
}

impl Error for MemberCalculatorError {}

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
pub struct MemberCalculator<'a> {
    continue_on_missing_items: bool,
    guild_id: GuildId,
    member_roles: &'a [&'a (RoleId, Permissions)],
    owner_id: Option<UserId>,
    user_id: UserId,
}

impl<'a> MemberCalculator<'a> {
    /// Create a calculator to calculate the permissions of a member.
    ///
    /// Using the returned member calculator, you can calculate the permissions
    /// of the member [across the guild][`permissions`] or
    /// [in a specified channel][`in_channel`].
    ///
    /// [`in_channel`]: struct.MemberCalculator.html#method.in_channel
    /// [`permissions`]: struct.MemberCalculator.html#method.permissions
    pub fn new(
        guild_id: GuildId,
        user_id: UserId,
        member_roles: &'a [&'a (RoleId, Permissions)],
    ) -> Self {
        Self {
            continue_on_missing_items: false,
            guild_id,
            owner_id: None,
            member_roles,
            user_id,
        }
    }

    /// Configure the ID of the owner of the guild.
    ///
    /// This should be used if you don't want to manually take the user ID and
    /// owner ID in account beforehand.
    pub fn owner_id(mut self, owner_id: UserId) -> Self {
        self.owner_id.replace(owner_id);

        self
    }

    /// Calculate the guild-level permissions of a member.
    ///
    /// # Errors
    ///
    /// If [`Calculator::continue_on_missing_items`] wasn't enabled, then this
    /// returns [`MemberCalculatorError::EveryoneRoleMissing`] if the `@everyone` role with the
    /// same ID as the guild wasn't found in the given guild roles map.
    ///
    /// [`Calculator::continue_on_missing_items`]: struct.Calculator.html#method.continue_on_missing_items
    /// [`MemberCalculatorError::EveryoneRoleMissing`]: enum.MemberCalculatorError.html#method.EveryoneRoleMissing
    pub fn root(&self) -> Result<Permissions, MemberCalculatorError> {
        // If the user is the owner, then we can just return all of the
        // permissions.
        if matches!(self.owner_id, Some(id) if id == self.user_id) {
            return Ok(Permissions::all());
        }

        // The permissions that the @everyone role has is the baseline.
        let mut permissions = if let Some(permissions) = self
            .member_roles
            .iter()
            .find(|role| (role.0).0 == self.guild_id.0)
        {
            permissions.1
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
                return Err(MemberCalculatorError::EveryoneRoleMissing {
                    guild_id: self.guild_id,
                });
            }
        };

        // Permissions on a user's roles are simply additive.
        for (_, role_permissions) in self.member_roles.iter() {
            if permissions.contains(Permissions::ADMINISTRATOR) {
                return Ok(Permissions::all());
            }

            permissions.insert(*role_permissions);
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
    /// [`Calculator::continue_on_missing_items`]: struct.Calculator.html#method.continue_on_missing_items
    /// [`Error::EveryoneRoleMissing`]: enum.Error.html#method.EveryoneRoleMissing
    /// [`permissions`]: #method.permissions
    pub fn in_channel<'b, U: IntoIterator<Item = &'b PermissionOverwrite> + Clone>(
        self,
        channel_type: ChannelType,
        channel_overwrites: U,
    ) -> Result<Permissions, MemberCalculatorError> {
        let mut permissions = self.root()?;

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

                    if !self.member_roles.iter().any(|(id, _)| *id == role) {
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
    use super::{GuildId, MemberCalculator, MemberCalculatorError, RoleId, UserId};
    use static_assertions::{assert_fields, assert_impl_all, assert_obj_safe};
    use std::{
        error::Error,
        fmt::{Debug, Display},
    };
    use twilight_model::{
        channel::{
            permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
            ChannelType,
        },
        guild::Permissions,
    };

    assert_fields!(MemberCalculatorError::EveryoneRoleMissing: guild_id);
    assert_impl_all!(
        MemberCalculatorError: Clone,
        Debug,
        Display,
        Error,
        Eq,
        PartialEq,
        Send,
        Sync
    );
    assert_impl_all!(MemberCalculator<'_>: Clone, Debug, Eq, PartialEq, Send, Sync);
    assert_obj_safe!(MemberCalculatorError, MemberCalculator<'_>);

    #[test]
    fn test_error_display() {
        assert_eq!(
            "@everyone role is missing for guild 123",
            MemberCalculatorError::EveryoneRoleMissing {
                guild_id: GuildId(123)
            }
            .to_string(),
        );
    }

    #[test]
    fn test_owner_is_admin() {
        let guild_id = GuildId(1);
        let user_id = UserId(2);
        let member_roles = &[&(RoleId(1), Permissions::SEND_MESSAGES)];

        let calculator = MemberCalculator::new(guild_id, user_id, member_roles).owner_id(user_id);

        assert_eq!(Permissions::all(), calculator.root().unwrap());
    }

    // Test that a permission overwrite denying the "View Channel" permission
    // implicitly denies all other permissions.
    #[test]
    fn test_view_channel_deny_implicit() {
        let guild_id = GuildId(1);
        let user_id = UserId(2);
        let member_roles = &[
            &(
                RoleId(1),
                Permissions::MENTION_EVERYONE | Permissions::SEND_MESSAGES,
            ),
            &(RoleId(3), Permissions::empty()),
        ];

        // First, test when it's denied for an overwrite on a role the user has.
        let overwrites = &[PermissionOverwrite {
            allow: Permissions::SEND_TTS_MESSAGES,
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Role(RoleId(3)),
        }];

        let calculated = MemberCalculator::new(guild_id, user_id, member_roles)
            .in_channel(ChannelType::GuildText, overwrites)
            .unwrap();

        assert_eq!(calculated, Permissions::empty());

        // And now that it's denied for an overwrite on the member.
        let overwrites = &[PermissionOverwrite {
            allow: Permissions::SEND_TTS_MESSAGES,
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Member(UserId(2)),
        }];

        let calculated = MemberCalculator::new(guild_id, user_id, member_roles)
            .in_channel(ChannelType::GuildText, overwrites)
            .unwrap();

        assert_eq!(calculated, Permissions::empty());
    }

    #[test]
    fn test_remove_text_perms_when_voice() {
        let guild_id = GuildId(1);
        let user_id = UserId(2);
        let member_roles = &[
            &(RoleId(1), Permissions::CONNECT),
            &(RoleId(3), Permissions::SEND_MESSAGES),
        ];

        let calculated = MemberCalculator::new(guild_id, user_id, member_roles)
            .in_channel(ChannelType::GuildVoice, &[])
            .unwrap();

        assert_eq!(calculated, Permissions::CONNECT);
    }

    #[test]
    fn test_remove_voice_perms_when_text() {
        let guild_id = GuildId(1);
        let user_id = UserId(2);
        let member_roles = &[
            &(RoleId(1), Permissions::CONNECT),
            &(RoleId(3), Permissions::SEND_MESSAGES),
        ];

        let calculated = MemberCalculator::new(guild_id, user_id, member_roles)
            .in_channel(ChannelType::GuildText, &[])
            .unwrap();

        assert_eq!(calculated, Permissions::SEND_MESSAGES);
    }

    // Test that denying the "Send Messages" permission denies all message
    // send related permissions.
    #[test]
    fn test_deny_send_messages_removes_related() {
        let guild_id = GuildId(1);
        let user_id = UserId(2);
        let member_roles = &[
            &(
                RoleId(1),
                Permissions::MANAGE_MESSAGES
                    | Permissions::EMBED_LINKS
                    | Permissions::MENTION_EVERYONE,
            ),
            &(RoleId(3), Permissions::empty()),
        ];

        // First, test when it's denied for an overwrite on a role the user has.
        let overwrites = &[PermissionOverwrite {
            allow: Permissions::ATTACH_FILES,
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(RoleId(3)),
        }];

        let calculated = MemberCalculator::new(guild_id, user_id, member_roles)
            .in_channel(ChannelType::GuildText, overwrites)
            .unwrap();

        assert_eq!(calculated, Permissions::MANAGE_MESSAGES);
    }
}
