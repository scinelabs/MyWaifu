use crate::{
    components::paginator::EmbedPaginator,
    utils::{fmt, ToEmbed},
    Context, Error,
};

#[derive(poise::ChoiceParameter)]
pub enum PackChoice {
    #[name = "Standard Pack"]
    StandardPack,
    #[name = "Gold Pack"]
    GoldPack,
}
impl PackChoice {
    pub fn waifu_count(&self) -> u32 {
        match self {
            Self::StandardPack => 3,
            Self::GoldPack => 5,
        }
    }
}

/// Summon a pack of waifus
#[poise::command(slash_command, check = "crate::checks::has_account")]
pub async fn summon(ctx: Context<'_>, pack: PackChoice) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let account = ctx.data().postgres.get_account(ctx.author().id).await?;
    let has_packs = match pack {
        PackChoice::StandardPack => account.packs > 0,
        PackChoice::GoldPack => account.premium_one_packs > 0,
    };
    if !has_packs {
        ctx.send(|cr| {
            cr.embed(|ce| {
                fmt::error(
                    "You don't have any packs of this type. Buy one in the pack shop (`/shop packs`)",
                    ce,
                )
            })
            .ephemeral(true)
        })
        .await?;
    } else {
        ctx.defer_ephemeral().await?;
        let waifus = ctx
            .data()
            .mongo
            .get_random_waifus(pack.waifu_count(), &account.waifus)
            .await?;
        let mut paginator = EmbedPaginator::new(waifus.clone());
        let selected_waifu = paginator.start(ctx, true).await?;
        if let Some(waifu) = selected_waifu {
            ctx.send(|cr| {
                cr.content(format!(
                    "**`{}`** has been added to your inventory.",
                    &waifu.name
                ))
                .embed(|ce| waifu.to_embed(ce))
                .ephemeral(true)
            })
            .await?;
        } else {
            ctx.send(|cr| {
                cr.ephemeral(true).embed(|ce| {
                    fmt::error(
                        "You did not select a waifu. A random one has been added to your inventory",
                        ce,
                    )
                })
            })
            .await?;
        }

        let (standard_packs, premium_packs) = match pack {
            PackChoice::StandardPack => (-1, 0),
            PackChoice::GoldPack => (0, -1),
        };

        ctx.data()
            .postgres
            .update_packs(ctx.author().id, standard_packs, premium_packs)
            .await?;

        let first = waifus.get(0).unwrap();
        let to_add = selected_waifu.unwrap_or(first);
        ctx.data()
            .postgres
            .add_waifu(ctx.author().id, to_add._id)
            .await?;
    }

    Ok(())
}

pub fn commands() -> [crate::Command; 1] {
    [summon()]
}
