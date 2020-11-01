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

Take a scenario where a member has two roles: the `@everyone` role (with the
same ID as the guild) that grants the View Channel permission across the
whole guild, and a second role that grants the Send Messages permission
across the whole guild. This means that, across the server, the member will
have the View Channel and Send Messages permissions, unless denied or
expanded by channel-specific permission overwrites.

In a given channel, there are two permission overwrites: one for the
`@everyone` role and one for the member itself. These overwrites look
like:

- `@everyone` role is allowed the Embed Links and Add Reactions permissions;
- Member is denied the Send Messages permission.

Taking into account the guild root-level permissions and the permission
overwrites, the end result is that in the specified channel the user has
the View Channel, Embed Links, and Add Reactions permission, but is denied
the Send Messages permission that their second role was granted on a root
level.

Let's see that in code:

```rust
use twilight_permission_calculator::Calculator;
use twilight_model::{
    channel::{
        permission_overwrite::{PermissionOverwriteType, PermissionOverwrite},
        ChannelType
    },
    guild::Permissions,
    id::{GuildId, RoleId, UserId},
};

let guild_id = GuildId(1);
let user_id = UserId(3);
let member_roles = &[
    // Guild-level @everyone role that, by default, allows everyone to view
    // channels.
    &(RoleId(1), Permissions::VIEW_CHANNEL),
    // Guild-level permission that grants members with the role the Send
    // Messages permission.
    &(RoleId(2), Permissions::SEND_MESSAGES),
];

let channel_overwrites = &[
    // All members are given the Add Reactions and Embed Links members via
    // the `@everyone` role.
    PermissionOverwrite {
        allow: Permissions::ADD_REACTIONS | Permissions::EMBED_LINKS,
        deny: Permissions::empty(),
        kind: PermissionOverwriteType::Role(RoleId(1)),
    },
    // Member is denied the Send Messages permission.
    PermissionOverwrite {
        allow: Permissions::empty(),
        deny: Permissions::SEND_MESSAGES,
        kind: PermissionOverwriteType::Member(user_id),
    },
];

let calculated_permissions = Calculator::new(guild_id, user_id, member_roles)
    .in_channel(ChannelType::GuildText, channel_overwrites)?;

// Now that we've got the member's permissions in the channel, we can
// check that they have the server-wide View Channel permission and
// the Add Reactions permission granted, but their guild-wide Send Messages
// permission was denied. Additionally, since the user can't send messages,
// their Embed Links permission was removed.

let expected = Permissions::ADD_REACTIONS | Permissions::VIEW_CHANNEL;
assert!(!calculated_permissions.contains(Permissions::EMBED_LINKS));
assert!(!calculated_permissions.contains(Permissions::SEND_MESSAGES));
assert_eq!(expected, calculated_permissions);
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
