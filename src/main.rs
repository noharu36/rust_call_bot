use anyhow::Context as _;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use std::sync::Arc;

use rust_callbot::{Handler, Settings};

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let settings = Settings {
        discord_token: secrets
            .get("DISCORD_TOKEN")
            .context("'DISCORD_TOKEN' was not found")?,
        guild_id: secrets
            .get("GUILD_ID")
            .context("'GUILD_ID' was not found")?
            .parse()
            .unwrap(),
        log_channel_id: secrets
            .get("LOG_CHANNEL_ID")
            .context("'LOG_CHANNEL_ID' was not found")?
            .parse()
            .unwrap(),
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

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
