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
twilight-permission-calculator = { branch = "main", git = "https://github.com/twilight-rs/permission-calculator" }
```

# Features

The `tracing` dependency is optional and can be disabled if you don't want
logging. To do this, use this in your `Cargo.toml`:

```toml
[dependencies]
twilight-permission-calculator = { branch = "main", default-features = false, git = "https://github.com/twilight-rs/permission-calculator" }
```

# Examples

## Calculating member permissions in a channel

Take a scenario where a guild with two members that are important here:
the owner and the normal user. There are 3 roles: the `@everyone` role
(with the same ID as the guild) that grants the "VIEW_CHANNEL"
permission across the whole guild, role ID 4 that grants the
"MANAGE_ROLES" permission across the whole guild, and role ID 5 that
grants the "SEND_MESSAGES" permission.

The normal user has, of course, the `@everyone` role, and additionally
role ID 5. This means that, across the server, the user will have the
"VIEW_CHANNEL" and "SEND_MESSAGES" permissions.

In a given channel, there are two permission overwrites:

- role ID 1 is not overwritten
- role ID 4 is allowed the "SEND_TTS_MESSAGES" permission, and isn't
denied any permissions
- role ID 5 is allowed the "MANAGE_MESSAGES" permission, but is denied
the "SEND_MESSAGES" permission

Taking into account the "VIEW_CHANNEL" permission granted to the
`@everyone` role across the guild, the "SEND_MESSAGES" permission
granted across the guild to those with role ID 5 (which the user has),
and that role ID 5 is allowed "MANAGE_MESSAGES" but is denied
"SEND_MESSAGES" in the channel, the user will have the "VIEW_CHANNEL"
and "MANAGE_MESSAGES" permission.

Let's see that in code:

```rust
use twilight_permission_calculator::{Calculator};
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
let guild_owner_id = UserId(2);
let user_id = UserId(6);
let member_roles = &[RoleId(5)];

let mut roles = HashMap::new();
// Insert the @everyone role that allows everyone to view channels.
roles.insert(RoleId(1), Permissions::VIEW_CHANNEL);

// And another role that the member doesn't have, but grants the
// "MANAGE_ROLES" permission in the guild as a whole.
roles.insert(RoleId(4), Permissions::MANAGE_ROLES);

// And another that the member *does* have, which grants the
// "SEND_MESSAGES" permission in the guild as a whole.
roles.insert(RoleId(5), Permissions::SEND_MESSAGES);

let channel_overwrites = &[
    PermissionOverwrite {
        allow: Permissions::SEND_TTS_MESSAGES,
        deny: Permissions::empty(),
        kind: PermissionOverwriteType::Role(RoleId(4)),
    },
    PermissionOverwrite {
        allow: Permissions::MANAGE_MESSAGES,
        deny: Permissions::SEND_MESSAGES,
        kind: PermissionOverwriteType::Role(RoleId(5)),
    },
];

let calculated_permissions = Calculator::new(guild_id, guild_owner_id, &roles)
    .member(user_id, member_roles)
    .in_channel(ChannelType::GuildText, channel_overwrites)?;

// Now that we've got the member's permissions in the channel, we can
// check that they have the server-wide "VIEW_CHANNEL" permission and
// the "MANAGE_MESSAGES" permission granted to the role in the channel,
// but their guild-wide "SEND_MESSAGES" permission was denied:

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
