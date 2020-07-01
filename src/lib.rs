//![![license badge][]][license link] [![rust badge]][rust link]
//!
//! ![project logo][logo]
//!
//! # rarity-permission-calculator
//!
//! `rarity-permission-calculator` is a permission calculator for the Discord
//! [`twilight-rs`] library.
//!
//! # Installation
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rarity-permission-calculator = { git = "https://github.com/rarity-rs/permission-calculator" }
//! ```
//!
//! # Features
//!
//! The `log` dependency is optional and can be disabled if you don't want
//! logging. To do this, use this in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rarity-permission-calculator = { default-features = false, git = "https://github.com/rarity-rs/permission-calculator" }
//! ```
//!
//! # Examples
//!
//! ## Calculating member permissions in a channel
//!
//! Take a scenario where a guild with two members that are important here:
//! the owner and the normal user. There are 3 roles: the `@everyone` role
//! (with the same ID as the guild) that grants the "VIEW_CHANNEL"
//! permission across the whole guild, role ID 4 that grants the
//! "MANAGE_ROLES" permission across the whole guild, and role ID 5 that
//! grants the "SEND_MESSAGES" permission.
//!
//! The normal user has, of course, the `@everyone` role, and additionally
//! role ID 5. This means that, across the server, the user will have the
//! "VIEW_CHANNEL" and "SEND_MESSAGES" permissions.
//!
//! In a given channel, there are two permission overwrites:
//!
//! - role ID 1 is not overwritten
//! - role ID 4 is allowed the "SEND_TTS_MESSAGES" permission, and isn't
//! denied any permissions
//! - role ID 5 is allowed the "MANAGE_MESSAGES" permission, but is denied
//! the "SEND_MESSAGES" permission
//!
//! Taking into account the "VIEW_CHANNEL" permission granted to the
//! `@everyone` role across the guild, the "SEND_MESSAGES" permission
//! granted across the guild to those with role ID 5 (which the user has),
//! and that role ID 5 is allowed "MANAGE_MESSAGES" but is denied
//! "SEND_MESSAGES" in the channel, the user will have the "VIEW_CHANNEL"
//! and "MANAGE_MESSAGES" permission.
//!
//! Let's see that in code:
//!
//! ```rust
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use rarity_permission_calculator::{Calculator, Role};
//! use std::collections::HashMap;
//! use twilight_model::{
//!     channel::permission_overwrite::{PermissionOverwriteType, PermissionOverwrite},
//!     guild::Permissions,
//!     id::{GuildId, RoleId, UserId},
//! };
//!
//! let guild_id = GuildId(1);
//! let guild_owner_id = UserId(2);
//! let user_id = UserId(6);
//! let member_roles = &[RoleId(5)];
//!
//! let mut roles = HashMap::new();
//! // Insert the @everyone role that allows everyone to view channels.
//! roles.insert(RoleId(1), Role::new(0, Permissions::VIEW_CHANNEL));
//!
//! // And another role that the member doesn't have, but grants the
//! // "MANAGE_ROLES" permission in the guild as a whole.
//! roles.insert(RoleId(4), Role::new(1, Permissions::MANAGE_ROLES));
//!
//! // And another that the member *does* have, which grants the
//! // "SEND_MESSAGES" permission in the guild as a whole.
//! roles.insert(RoleId(5), Role::new(2, Permissions::SEND_MESSAGES));
//!
//! let channel_overwrites = &[
//!     PermissionOverwrite {
//!         allow: Permissions::SEND_TTS_MESSAGES,
//!         deny: Permissions::empty(),
//!         kind: PermissionOverwriteType::Role(RoleId(4)),
//!     },
//!     PermissionOverwrite {
//!         allow: Permissions::MANAGE_MESSAGES,
//!         deny: Permissions::SEND_MESSAGES,
//!         kind: PermissionOverwriteType::Role(RoleId(5)),
//!     },
//! ];
//!
//! let calculated_permissions = Calculator::new(guild_id, guild_owner_id, &roles)
//!     .member(user_id, member_roles)
//!     .in_channel(channel_overwrites)?;
//!
//! // Now that we've got the member's permissions in the channel, we can
//! // check that they have the server-wide "VIEW_CHANNEL" permission and
//! // the "MANAGE_MESSAGES" permission granted to the role in the channel,
//! // but their guild-wide "SEND_MESSAGES" permission was denied:
//!
//! let expected = Permissions::MANAGE_MESSAGES | Permissions::VIEW_CHANNEL;
//! assert_eq!(expected, calculated_permissions);
//! assert!(!calculated_permissions.contains(Permissions::SEND_MESSAGES));
//! # Ok(()) }
//! ```
//!
//! [`twilight-rs`]: https://github.com/twilight-rs/twilight
//! [license badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=flat-square
//! [license link]: https://opensource.org/licenses/ISC
//! [logo]: https://raw.githubusercontent.com/rarity-rs/assets/main/logo.png
//! [rust badge]: https://img.shields.io/badge/rust-1.44.1+-93450a.svg?style=flat-square
//! [rust link]: https://blog.rust-lang.org/2020/06/18/Rust.1.44.1.html

#![doc(html_logo_url = "https://raw.githubusercontent.com/rarity-rs/assets/main/logo.png")]
#![deny(
    clippy::all,
    clippy::pedantic,
    future_incompatible,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    unsafe_code,
    unused,
    warnings
)]

pub mod prelude;

use std::{
    collections::HashMap,
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    channel::permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
    guild::Permissions,
    id::{GuildId, RoleId, UserId},
};

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

/// Basic information about a guild role's guild-level granted permissions and
/// position in the role list.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use = "roles must be given to the calculator"]
pub struct Role {
    permissions: Permissions,
    position: i64,
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

/// A calculator to calculate permissions of various things within in a guild.
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
    pub fn member(self, user_id: UserId, member_role_ids: &'a [RoleId]) -> MemberCalculator<'a> {
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
#[must_use = "the member calculator isn't useful if you don't calculate permissions"]
pub struct MemberCalculator<'a> {
    continue_on_missing_items: bool,
    guild_id: GuildId,
    guild_owner_id: UserId,
    member_role_ids: &'a [RoleId],
    roles: &'a HashMap<RoleId, Role>,
    user_id: UserId,
}

impl<'a> MemberCalculator<'a> {
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
        for role_id in self.member_role_ids {
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
    pub fn in_channel(
        self,
        channel_overwrites: &'a [PermissionOverwrite],
    ) -> Result<Permissions, Error> {
        let mut permissions = self.permissions()?;

        let mut data = Vec::new();

        for overwrite in channel_overwrites {
            if let PermissionOverwriteType::Role(role) = overwrite.kind {
                if role.0 != self.guild_id.0
                    && !self.member_role_ids.iter().any(|r| *r == role)
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
