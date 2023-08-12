use std::borrow::Cow;

use petgraph::{
    dot::{Config as DotConfig, Dot},
    Graph,
};
use poise::serenity_prelude as serenity;
use tokio::io::AsyncWriteExt;

use crate::{
    components::confirm::ConfirmMenu,
    utils::{fmt, random_component_id},
    Context, Error,
};

#[poise::command(
    slash_command,
    subcommands("visualize", "create", "invite", "delete"),
    check = "crate::checks::has_account"
)]
pub async fn alliance(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Create your own alliance
#[poise::command(slash_command)]
pub async fn create(ctx: Context<'_>, name: String) -> Result<(), Error> {
    if name.len() > 15 {
        ctx.send(|cr| {
            cr.embed(|ce| fmt::error("Alliance name must be below 15 characters long.", ce))
        })
        .await?;
        return Ok(());
    }
    ctx.defer_ephemeral().await?;

    ctx.data()
        .postgres
        .create_alliance(ctx.author().id, &name)
        .await?;
    ctx.send(|cr| {
        cr.embed(|ce| {
            fmt::success(
                "Alliance created! Check it out with `/alliance visualize`",
                ce,
            )
        })
    })
    .await?;

    Ok(())
}

/// Delete your alliance
#[poise::command(slash_command, check = "crate::checks::in_alliance")]
pub async fn delete(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let alliance = ctx.data().postgres.get_alliance(ctx.author().id).await?;
    if alliance.owner != ctx.author().id.0 as i64 {
        ctx.send(|cr| cr.embed(|ce| fmt::error("You must own an alliance to delete it!", ce)))
            .await?;
    } else {
        ctx.data().postgres.delete_alliance(ctx.author().id).await?;
        ctx.data()
            .check_cache
            .insert_in_alliance(ctx.author().id, false)
            .await;
        ctx.send(|cr| cr.embed(|ce| fmt::success("Alliance deleted.", ce)))
            .await?;
    }

    Ok(())
}

#[poise::command(slash_command, check = "crate::checks::in_alliance")]
pub async fn invite(ctx: Context<'_>, member: serenity::Member) -> Result<(), Error> {
    ctx.defer().await?;

    let member_account = ctx.data().postgres.get_account(member.user.id).await;
    if member_account.is_err() {
        ctx.send(|cr| cr.embed(|ce| fmt::error("This user does not have an account. Tell them to make one to invite them to your alliance", ce)).ephemeral(true)).await?;
        return Ok(());
    } else {
        let alliance_result = ctx.data().postgres.get_alliance(member.user.id).await;
        if alliance_result.is_ok() {
            ctx.send(|cr| cr.embed(|ce| fmt::error("This user is already in an alliance. Ask them to leave it if you want them to join yours", ce)).ephemeral(true)).await?;
            return Ok(());
        }
    }

    let alliance = ctx.data().postgres.get_alliance(ctx.author().id).await?;
    if alliance.owner != ctx.author().id.0 as i64 {
        ctx.send(|cr| {
            cr.embed(|ce| fmt::error("You must own the alliance to invite people", ce))
                .ephemeral(true)
        })
        .await?;
    } else {
        let message = format!(
            "**`{}`**, would you like to join **`{}`**'s alliance?",
            member.display_name(),
            ctx.author().name
        );
        let confirmed = ConfirmMenu::start(ctx, &message).await?;
        if confirmed {
            ctx.data()
                .postgres
                .join_alliance(ctx.author().id, member.user.id)
                .await?;
            ctx.send(|cr| {
                cr.embed(|ce| {
                    fmt::success(
                        "Joined alliance. Check out your new alliance with `/alliance visualize`",
                        ce,
                    )
                })
            })
            .await?;
        }
    }

    Ok(())
}

/// Visualize your alliance in the form of a tree
#[poise::command(slash_command, check = "crate::checks::in_alliance")]
pub async fn visualize(ctx: Context<'_>) -> Result<(), Error> {
    // TODO: optimise
    ctx.defer_ephemeral().await?;

    let alliance = ctx.data().postgres.get_alliance(ctx.author().id).await?;

    let mut graph = Graph::<&str, &str>::new();

    let mut previous_member_node = None;
    let mut name_bindings: Vec<(i64, String)> = vec![];
    let mut user_waifu_names: Vec<(String, Vec<String>)> = vec![];
    let mut pairs = vec![];

    let mut full_alliance_members = alliance.members.clone();
    full_alliance_members.push(alliance.owner);

    for (idx, user_id) in full_alliance_members.iter().enumerate() {
        name_bindings.push((user_id.clone(), format!("Member {idx}")))
    }

    for (user_id, member_string) in name_bindings.iter() {
        let waifu_ids = ctx
            .data()
            .postgres
            .get_waifus(serenity::UserId(user_id.clone() as u64))
            .await?;
        let waifus = ctx
            .data()
            .mongo
            .get_waifus(waifu_ids.iter().map(|el| el.clone() as i32).collect())
            .await?;

        let waifu_names: Vec<String> = waifus.iter().map(|w| w.name.clone()).collect();
        user_waifu_names.push((member_string.clone(), waifu_names));
    }

    for (member_string, waifu_names) in user_waifu_names.iter() {
        let mut inner = vec![];
        let member_node = graph.add_node(member_string);
        for w_name in waifu_names.iter() {
            let w_node = graph.add_node(w_name);
            inner.push((member_node, w_node));
        }
        if let Some(previous_m_node) = previous_member_node {
            inner.push((previous_m_node, member_node))
        }

        previous_member_node = Some(member_node);

        pairs.append(&mut inner);
    }

    graph.extend_with_edges(pairs);

    let dot_config = [DotConfig::EdgeNoLabel];
    let notation = Dot::with_config(&graph, &dot_config).to_string();
    let graph_id = random_component_id();
    let path = format!("stages/ats/{graph_id}.dot");
    let mut file = tokio::fs::File::create(&path).await?;
    file.write(notation.as_bytes()).await?;
    file.flush().await?;

    let image_path = format!("stages/ats/{graph_id}.png");
    async_process::Command::new("dot")
        .arg("-Kfdp")
        .arg("-Tpng")
        .arg(&path)
        .arg("-o")
        .arg(&image_path)
        .output()
        .await?;

    let image_file = tokio::fs::read(&image_path).await?;
    let image_data = Cow::from(&image_file);
    let attachment = serenity::AttachmentType::Bytes {
        data: image_data,
        filename: String::from("graph.png"),
    };

    ctx.send(|cr| {
        cr.embed(|ce| {
            ce.title(&alliance.name)
                .image("attachment://graph.png")
                .description(format!(
                    "This alliance has {} members",
                    alliance.members.len() + 1 // to include the owner
                ))
                .colour(serenity::Colour::BLITZ_BLUE)
        })
        .attachment(attachment)
    })
    .await?;

    tokio::fs::remove_file(&image_path).await?;
    tokio::fs::remove_file(&path).await?;

    Ok(())
}

pub fn commands() -> [crate::Command; 1] {
    [alliance()]
}
