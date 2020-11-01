use criterion::{criterion_group, criterion_main, Criterion};
use twilight_model::{
    channel::{
        permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
        ChannelType,
    },
    guild::Permissions,
    id::{GuildId, RoleId, UserId},
};
use twilight_permission_calculator::Calculator;

fn member_calculator_in_channel() {
    let guild_id = GuildId(1);
    let guild_owner_id = UserId(2);
    let member_roles = &[
        &(RoleId(1), Permissions::VIEW_CHANNEL),
        &(RoleId(3), Permissions::SEND_MESSAGES),
    ];

    let channel_overwrites = &[PermissionOverwrite {
        allow: Permissions::MANAGE_MESSAGES,
        deny: Permissions::SEND_MESSAGES,
        kind: PermissionOverwriteType::Role(RoleId(3)),
    }];

    let calculated_permissions = Calculator::new(guild_id, guild_owner_id, member_roles)
        .in_channel(ChannelType::GuildText, channel_overwrites)
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
