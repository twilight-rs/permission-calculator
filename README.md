<!-- cargo-sync-readme start -->

[![discord badge][]][discord link] [![github badge][]][github link] [![license badge][]][license link] [![rust badge]][rust link]

![project logo][logo]

# twilight-permission-calculator

`twilight-permission-calculator` is a permission calculator for the Discord
[`twilight-rs`] library.

# Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
twilight-permission-calculator = { branch = "trunk", git = "https://github.com/twilight-rs/permission-calculator" }
```

# Features

The `tracing` dependency is optional and can be disabled if you don't want
logging. To do this, use this in your `Cargo.toml`:

```toml
[dependencies]
twilight-permission-calculator = { branch = "trunk", default-features = false, git = "https://github.com/twilight-rs/permission-calculator" }
```

# Examples

## Calculating member permissions in a channel

Take a scenario where a guild with two members that are important here:
the owner and the normal user. There are 3 roles: the `@everyone` role
(with the same ID as the guild) that grants the View Channel permission
across the whole guild, role ID 4 that grants the Manage Roles permission
across the whole guild, and role ID 5 that grants the Send Messages
permission.

The normal user has, of course, the `@everyone` role, and additionally role
ID 3. This means that, across the server, the user will have the View
Channel and Send Messages permissions.

In a given channel, there are two permission overwrites:

- role ID 1 is not overwritten
- role ID 3 is allowed the Manage Messages permission, but is denied the
Send Messages permission

Taking into account the View Channel permission granted to the `@everyone`
role across the guild, the Send Messages permission granted across the guild
to those with role ID 5 (which the user has), and that role ID 5 is allowed
Manage Messages but is denied Send Messages in the channel, the user will
have the View Channel and Manage Messages permission.

Let's see that in code:

```rust
use twilight_permission_calculator::Calculator;
use std::collections::HashMap;
use twilight_model::{
    channel::{
        permission_overwrite::{PermissionOverwriteType, PermissionOverwrite},
        ChannelType
    },
    guild::Permissions,
    id::{GuildId, RoleId, UserId},
};

let guild_id = GuildId(1);
let user_id = UserId(4);
let member_roles = &[
    // Guild-level @everyone role that, by default, allows everyone to view
    // channels.
    &(RoleId(1), Permissions::VIEW_CHANNEL),
    // Guild-level permission that grants everyone the Send Messages
    // permission by default.
    &(RoleId(3), Permissions::SEND_MESSAGES),
];

let channel_overwrites = &[
    PermissionOverwrite {
        allow: Permissions::SEND_TTS_MESSAGES,
        deny: Permissions::empty(),
        kind: PermissionOverwriteType::Role(RoleId(2)),
    },
    PermissionOverwrite {
        allow: Permissions::MANAGE_MESSAGES,
        deny: Permissions::SEND_MESSAGES,
        kind: PermissionOverwriteType::Role(RoleId(3)),
    },
];

let calculated_permissions = Calculator::new(guild_id, user_id, member_roles)
    .in_channel(ChannelType::GuildText, channel_overwrites)?;

// Now that we've got the member's permissions in the channel, we can
// check that they have the server-wide View Channel permission and
// the Manage Messages permission granted to the role in the channel,
// but their guild-wide Send Messages permission was denied:

let expected = Permissions::MANAGE_MESSAGES | Permissions::VIEW_CHANNEL;
assert_eq!(expected, calculated_permissions);
assert!(!calculated_permissions.contains(Permissions::SEND_MESSAGES));
```

[`twilight-rs`]: https://github.com/twilight-rs/twilight
[license badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=for-the-badge
[license link]: https://opensource.org/licenses/ISC
[logo]: https://raw.githubusercontent.com/twilight-rs/twilight/trunk/logo.png
[rust badge]: https://img.shields.io/badge/rust-1.44.1+-93450a.svg?style=for-the-badge
[rust link]: https://blog.rust-lang.org/2020/06/18/Rust.1.44.1.html
[discord badge]: https://img.shields.io/discord/745809834183753828?color=%237289DA&label=discord%20server&logo=discord&style=for-the-badge
[discord link]: https://discord.gg/7jj8n7D
[github badge]: https://img.shields.io/badge/github-twilight-6f42c1.svg?style=for-the-badge&logo=github
[github link]: https://github.com/twilight-rs/twilight

<!-- cargo-sync-readme end -->
