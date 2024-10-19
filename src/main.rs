mod render;

use poise::{samples::register_globally, Framework, FrameworkOptions, PrefixFrameworkOptions};
use render::render;
use serenity::all::{ClientBuilder, GatewayIntents};
use std::env;

const COMMAND_PREFIX: &str = "~";

type Error = Box<dyn std::error::Error + Send + Sync>;

struct UserData;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN environment variable");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let commands = vec![render()];

    let framework_options = FrameworkOptions {
        commands,
        prefix_options: PrefixFrameworkOptions {
            prefix: Some(COMMAND_PREFIX.into()),
            ..PrefixFrameworkOptions::default()
        },
        ..FrameworkOptions::default()
    };

    let framework = Framework::builder()
        .options(framework_options)
        .setup(|context, _, framework| {
            Box::pin(async move {
                register_globally(context, &framework.options().commands).await?;
                Result::<UserData, Error>::Ok(UserData)
            })
        })
        .build();

    ClientBuilder::new(token, intents)
        .framework(framework)
        .await?
        .start()
        .await?;
    Ok(())
}
