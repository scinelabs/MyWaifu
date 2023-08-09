mod accounts;
mod alliances;
mod interactions;
mod shop;
mod summon;

use crate::{components::paginator::EmbedPaginator, utils::fmt};

pub fn commands() -> Vec<crate::Command> {
    accounts::commands()
        .into_iter()
        .chain(summon::commands())
        .chain(shop::commands())
        .chain(interactions::commands())
        .chain(alliances::commands())
        .chain([hello(), search()])
        .collect()
}

/// Make the bot say hi. Simply a debug command.
#[poise::command(slash_command)]
pub async fn hello(ctx: crate::Context<'_>) -> Result<(), crate::Error> {
    ctx.say("Hello.").await?;

    Ok(())
}

/// Search for waifus. Useful for checking if a waifu exists
#[poise::command(slash_command)]
pub async fn search(ctx: crate::Context<'_>, name: String) -> Result<(), crate::Error> {
    ctx.defer_ephemeral().await?;
    let waifus = ctx.data().mongo.search_waifus(&name).await?;
    if waifus.len() <= 0 {
        ctx.send(|cr| cr.embed(|ce| fmt::error("No results found with that name.", ce)))
            .await?;
    } else {
        let mut paginator = EmbedPaginator::new(waifus);
        paginator.start(ctx, false).await?;
    }

    Ok(())
}
