use poise::serenity_prelude as serenity;

use crate::{
    components::{confirm::ConfirmMenu, paginator::EmbedPaginator},
    utils::fmt,
    Context, Error,
};

pub const ACCOUNT_ERROR_MESSAGE: &str = "An error occurred. You probably already have an account. View your account with `/account view`\n\
If you do not have an account, join our support server and view our `#outages` channel, or contact support.";

/// Account related commands
#[poise::command(slash_command, subcommands("create", "view", "delete", "waifus"))]
pub async fn account(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Create an account
#[poise::command(slash_command)]
pub async fn create(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let register_result = ctx.data().postgres.register_account(ctx.author().id).await;
    if register_result.is_ok() {
        ctx.data()
            .check_cache
            .insert_has_account(ctx.author().id, true)
            .await;
        ctx.send(|cr| cr.embed(|ce| fmt::success("Account registered. You've been given 3 free packs and 500 currency to summon your first waifus!", ce))).await?;
    } else {
        ctx.send(|cr| cr.embed(|ce| fmt::error(ACCOUNT_ERROR_MESSAGE, ce)))
            .await?;
    }

    Ok(())
}

/// Delete your MyWaifu account
#[poise::command(slash_command, check = "crate::checks::has_account")]
pub async fn delete(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let confirmed = ConfirmMenu::start(
        ctx,
        ctx.author().id,
        "Are you sure you want to delete your MyWaifu! account? You'll lose all your waifus and currency. Your account **cannot** be recovered."
    ).await?;

    if confirmed {
        ctx.data().postgres.delete_account(ctx.author().id).await?;
        ctx.data()
            .check_cache
            .insert_has_account(ctx.author().id, false)
            .await;
        ctx.send(|cr| {
            cr.embed(|ce| {
                fmt::success(
                    "Deleted account. You can make a new one with `/account create`",
                    ce,
                )
            })
            .ephemeral(true)
        })
        .await?;
    } else {
        ctx.send(|cr| {
            cr.ephemeral(true)
                .embed(|ce| fmt::error("Cancelled account deletion.", ce))
        })
        .await?;
    }

    Ok(())
}

/// View your account
#[poise::command(slash_command, check = "crate::checks::has_account")]
pub async fn view(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let account = ctx.data().postgres.get_account(ctx.author().id).await?;
    let level = account.experience / 250;
    ctx.send(|cr| {
        cr.embed(|ce| {
            ce.author(|ca| {
                ca.name(format!("{}'s Account", ctx.author().name))
                    .icon_url(
                        ctx.author()
                            .avatar_url()
                            .unwrap_or(ctx.author().default_avatar_url()),
                    )
            })
            .field("Packs", account.packs, true)
            .field("Gold Packs", account.premium_one_packs, true)
            .field("Currency", account.currency, true)
            .field("Waifu Count", account.waifus.len(), true)
            .field("Level", level, true)
            .field("Experience", account.experience, true)
            .thumbnail(
                ctx.author()
                    .avatar_url()
                    .unwrap_or(ctx.author().default_avatar_url()),
            )
            .colour(serenity::Colour::FABLED_PINK)
        })
    })
    .await?;

    Ok(())
}

/// View your waifus
#[poise::command(slash_command, check = "crate::checks::has_account")]
pub async fn waifus(
    ctx: Context<'_>,
    #[description = "Whether only YOU should see the menu"] ephemeral: bool,
) -> Result<(), Error> {
    match ephemeral {
        true => ctx.defer_ephemeral().await?,
        false => ctx.defer().await?,
    };
    let waifu_ids = ctx.data().postgres.get_waifus(ctx.author().id).await?;
    let transformed: Vec<i32> = waifu_ids.iter().map(|id| id.clone().into()).collect();
    let waifus = ctx.data().mongo.get_waifus(transformed).await?;
    if waifus.len() <= 0 {
        ctx.send(|cr| cr.embed(|ce| fmt::error("You don't have any waifus.", ce)))
            .await?;
    } else {
        let mut paginator = EmbedPaginator::new(waifus);
        paginator.start(ctx, false).await?;
    }

    Ok(())
}

pub fn commands() -> [crate::Command; 1] {
    [account()]
}
