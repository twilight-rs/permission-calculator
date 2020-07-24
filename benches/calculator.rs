use criterion::{criterion_group, criterion_main, Criterion};
use rarity_permission_calculator::Calculator;
use std::collections::HashMap;
use twilight_model::{
    channel::permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
    guild::Permissions,
    id::{GuildId, RoleId, UserId},
};

fn member_calculator_in_channel() {
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
        .in_channel(channel_overwrites)
        .unwrap();

    // Now that we've got the member's permissions in the channel, we can
    // check that they have the server-wide "VIEW_CHANNEL" permission and
    // the "MANAGE_MESSAGES" permission granted to the role in the channel,
    // but their guild-wide "SEND_MESSAGES" permission was denied:

    let expected = Permissions::MANAGE_MESSAGES | Permissions::VIEW_CHANNEL;
    assert_eq!(expected, calculated_permissions);
    assert!(!calculated_permissions.contains(Permissions::SEND_MESSAGES));
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("member calculator - in channel", |b| {
        b.iter(member_calculator_in_channel)
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
