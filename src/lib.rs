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
//! rarity-permission-calculator = { branch = "main", git = "https://github.com/rarity-rs/permission-calculator" }
//! ```
//!
//! # Features
//!
//! The `log` dependency is optional and can be disabled if you don't want
//! logging. To do this, use this in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rarity-permission-calculator = { branch = "main", default-features = false, git = "https://github.com/rarity-rs/permission-calculator" }
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
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use rarity_permission_calculator::Calculator;
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
//! roles.insert(RoleId(1), Permissions::VIEW_CHANNEL);
//!
//! // And another role that the member doesn't have, but grants the
//! // "MANAGE_ROLES" permission in the guild as a whole.
//! roles.insert(RoleId(4), Permissions::MANAGE_ROLES);
//!
//! // And another that the member *does* have, which grants the
//! // "SEND_MESSAGES" permission in the guild as a whole.
//! roles.insert(RoleId(5), Permissions::SEND_MESSAGES);
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
    future_incompatible,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    unsafe_code,
    unused,
    warnings
)]

pub mod prelude;

mod calculator;
mod error;

pub use self::{
    calculator::{Calculator, MemberCalculator},
    error::Error,
};
