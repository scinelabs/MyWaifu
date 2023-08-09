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
    subcommands("visualize", "create", "invite"),
    check = "crate::checks::has_account",
    check = "crate::checks::in_alliance"
)]
pub async fn alliance(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Create your own alliance
#[poise::command(slash_command)]
pub async fn create(ctx: Context<'_>, name: String) -> Result<(), Error> {
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

#[poise::command(slash_command)]
pub async fn invite(ctx: Context<'_>, member: serenity::Member) -> Result<(), Error> {
    let message = format!(
        "**`{}`**, would you like to join **`{}`**'s alliance?",
        member.display_name(),
        ctx.author().name
    );
    let confirm_menu = ConfirmMenu::start(ctx, &message).await?;

    Ok(())
}

/// Visualize your alliance in the form of a tree
#[poise::command(slash_command)]
pub async fn visualize(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let alliance_result = ctx.data().postgres.get_alliance(ctx.author().id).await;

    if let Ok(alliance) = alliance_result {
        let account = ctx.data().postgres.get_account(ctx.author().id).await?;
        let waifus = ctx
            .data()
            .mongo
            .get_waifus(account.waifus.iter().map(|x| x.clone() as i32).collect())
            .await?;

        let mut graph = Graph::<&str, &str>::new();
        let names = vec!["mooon"];

        let mut pairs = vec![];

        for name in names.iter() {
            let mut inner = vec![];
            let name_node = graph.add_node(name.to_owned());

            for waifu in waifus.iter() {
                let waifu_node = graph.add_node(&waifu.name);
                inner.push((name_node, waifu_node));
            }
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
    } else {
        ctx.send(|cr| cr.embed(|ce| fmt::error("You're not in a alliance. Join one with `/alliance join` or make one with `/alliance create`", ce))).await?;
    }

    Ok(())
}

pub fn commands() -> [crate::Command; 1] {
    [alliance()]
}
