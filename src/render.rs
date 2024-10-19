use std::sync::Arc;

use mathjax::{MathJax, Render};
use poise::CreateReply;
use rand::random;
use serenity::all::CreateAttachment;
use tokio::{fs::remove_file, task::spawn_blocking, time::Instant};

use crate::{Error, UserData};

type Context<'a> = poise::Context<'a, UserData, Error>;

#[poise::command(slash_command)]
pub async fn render(
    context: Context<'_>,
    #[description = "LaTeX expression"] expression: String,
    #[description = "Fill color"] color: Option<String>,
) -> Result<(), Error> {
    render_image(
        context,
        MathJax::new().expect("MathJax renderer"),
        expression,
        color,
    )
    .await?;

    Ok(())
}

async fn render_image(
    context: Context<'_>,
    renderer: MathJax,
    expression: String,
    color: Option<String>,
) -> Result<(), Error> {
    let render_start_time = Instant::now();

    match renderer.render(expression) {
        Ok(mut image) => {
            context.defer().await?;
            let file_path = Arc::new(generate_file_path());

            set_image_color(&mut image, color);
            save_image(image, Arc::clone(&file_path)).await?;

            let reply = CreateReply::default()
                .content(generate_render_time_message(render_start_time))
                .attachment(CreateAttachment::path(&*file_path).await?);

            remove_file(&*file_path).await?;
            context.send(reply).await?;
        }

        Err(_) => {
            context.defer_ephemeral().await?;
            context.say(String::from("Invalid expression")).await?;
        }
    }

    Ok(())
}

fn generate_file_path() -> String {
    format!("math{}.png", random::<u32>())
}

fn set_image_color(image: &mut Render, color: Option<String>) {
    image.set_color(&color.unwrap_or(String::from("#FFAAAA")));
}

async fn save_image(image: Render, file_path: Arc<String>) -> Result<(), Error> {
    let file_path_ref = Arc::clone(&file_path);
    let save_closure = move || {
        image.into_image(20.0)?.save(&*file_path_ref)?;
        Result::<(), Error>::Ok(())
    };
    spawn_blocking(save_closure).await??;

    Ok(())
}

fn generate_render_time_message(instant: Instant) -> String {
    format!("-# Rendered in {:?}", instant.elapsed())
}
