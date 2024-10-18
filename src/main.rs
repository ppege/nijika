use mathjax::MathJax;
use poise::{serenity_prelude as serenity, CreateReply};
use rand::{random, seq::SliceRandom, Rng};
use serenity::all::CreateAttachment;
use std::{thread, time};

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Renders a LaTeX expression
#[poise::command(slash_command, prefix_command)]
async fn render(
    ctx: Context<'_>,
    #[description = "LaTeX expression"] expression: String,
    #[description = "Fill color"] color: Option<String>,
) -> Result<(), Error> {
    let now = time::Instant::now();
    let renderer = MathJax::new().unwrap();
    match renderer.render(expression) {
        Ok(mut image) => {
            ctx.defer().await?;
            image.set_color(&color.unwrap_or(String::from("#FFAAAA")));
            let file_path = format!("math{:5}.png", random::<u32>()).replace(" ", "0");
            image.into_image(20.0)?.save(&file_path)?;
            let content = format!("-# Rendered in {:?}", now.elapsed());
            let builder = CreateReply::default()
                .content(content)
                .attachment(CreateAttachment::path(&file_path).await?);
            std::fs::remove_file(&file_path)?;
            ctx.send(builder).await?;
        }
        Err(_) => {
            ctx.defer_ephemeral().await?;
            ctx.say(String::from("Invalid expression")).await?;
        }
    };
    Ok(())
}

async fn channel_message_loop(
    ctx: impl AsRef<serenity::Http>,
    channel_id: u64,
    guild_id: u64,
) -> Result<(), serenity::Error> {
    let channel = serenity::ChannelId::new(channel_id);
    let guild = serenity::GuildId::new(guild_id);
    let member_list = guild.members(ctx.as_ref(), None, None).await?;
    loop {
        let random_member = member_list
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_owned();
        let user = random_member.user;
        let name = user.global_name.unwrap_or(user.name);
        let content = format!("Welcome to {}'s room.", name);
        channel.say(ctx.as_ref(), content).await?;
        let seconds = rand::thread_rng().gen_range(3600..=7200);
        thread::sleep(time::Duration::from_secs(seconds));
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![render()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            tokio::spawn(channel_message_loop(
                ctx.clone(),
                1271013618271391785,
                1237186067665260625,
            ));
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
