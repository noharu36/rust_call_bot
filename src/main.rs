use anyhow::Context as _;
use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    model::{gateway::Ready, id::ChannelId, voice::VoiceState, colour::Colour, Timestamp},
    prelude::*,
};
use shuttle_runtime::SecretStore;
use std::sync::Arc;
use chrono::FixedOffset;
use tracing::info;

#[derive(PartialEq)]
enum Status {
    Joined,
    Leaved,
    Other,
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        let data = ctx.data.read().await;
        if let Some(data) = data.get::<Settings>() {
            let Some(guild_id) = new.guild_id else {
                println!("guild_id err");
                return;
            };

            let data = data.clone();
            if guild_id != data.guild_id {
                return;
            }

            let (status, _) = if let Some(old) = old {
                if let None = &new.channel_id {
                    (Status::Leaved, old.channel_id)
                } else {
                    (Status::Other, old.channel_id)
                }
            } else {
                (Status::Joined, new.channel_id)
            };

            if status == Status::Other {
                    return;
            }

            let user_name = if let Some(u) = &new.member {
                if let Some(nick_name) = u.user.nick_in(&ctx, guild_id).await {
                    nick_name.clone()
                } else {
                    u.display_name().to_string()
                }
            } else {
                "Unknown user".to_string()
            };

            let ch = ChannelId::new(data.log_channel_id);

            let embed = CreateEmbed::new()
                .title("Voice Channel Notice")
                .description({
                    if status == Status::Joined {
                        format!("**{}** がVCに入りました", user_name)
                    } else {
                        format!("**{}** がVCから抜けました", user_name)
                    }})
                .color({
                    if status == Status::Joined {
                    Colour(0x2aed24)
                    } else {
                    Colour(0xed2424)
                    }})
                .timestamp(
                    Timestamp::now()
                    .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap())
                );
            let message = CreateMessage::new().embed(embed);

            if let Err(e) = ch.send_message(&ctx.http, message).await {
                println!("ERROR: failed to send an message => {}", e);
            }
        }
    }
}

#[derive(Debug)]
struct Settings {
    discord_token: String,
    guild_id: u64,
    log_channel_id: u64,
}

impl TypeMapKey for Settings {
    type Value = Arc<Settings>;
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let settings = Settings {
        discord_token: secrets.get("DISCORD_TOKEN").context("'DISCORD_TOKEN' was not found")?,
        guild_id: secrets.get("GUILD_ID").context("'GUILD_ID' was not found")?.parse().unwrap(),
        log_channel_id: secrets.get("LOG_CHANNEL_ID").context("'LOG_CHANNEL_ID' was not found")?.parse().unwrap(),
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES;

    let client = Client::builder(&settings.discord_token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<Settings>(Arc::new(settings));
    }

    Ok(client.into())
}
