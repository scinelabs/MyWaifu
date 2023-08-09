use rand::{thread_rng, Rng};

use crate::{utils::fmt, Context, Error};

#[derive(Debug, poise::ChoiceParameter)]
pub enum FoodChoice {
    #[name = "🍞 Bread - 100 🪙"]
    Bread,
    #[name = "🍨 Ice Cream - 200 🪙"]
    IceCream,
    #[name = "🍖 Meat - 475 🪙"]
    Meat,
}
impl FoodChoice {
    pub fn icon<'a>(&self) -> &'a str {
        match self {
            Self::Bread => "🍞",
            Self::IceCream => "🍨",
            Self::Meat => "🍖",
        }
    }
    pub fn price(&self) -> u16 {
        match self {
            Self::Bread => 100,
            Self::IceCream => 200,
            Self::Meat => 475,
        }
    }
    pub fn random_experience(&self) -> u16 {
        let range = match self {
            Self::Bread => 10..=20,
            Self::IceCream => 20..=35,
            Self::Meat => 100..=125,
        };
        let mut rng = thread_rng();
        let num = rng.gen_range(range);

        num
    }
}

#[poise::command(
    slash_command,
    subcommands("feed", "sell"),
    check = "crate::checks::has_account"
)]
pub async fn interact(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

pub async fn autocomplete_waifu_name<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Iterator<Item = poise::AutocompleteChoice<u16>> {
    let waifu_ids = ctx
        .data()
        .postgres
        .get_waifus(ctx.author().id)
        .await
        .unwrap_or(vec![]);
    let waifu_ids = waifu_ids.iter().map(|el| el.clone() as i32).collect();
    let waifus = ctx
        .data()
        .mongo
        .get_waifus(waifu_ids)
        .await
        .unwrap_or(vec![]);
    let mut possible_options = vec![];

    for waifu in waifus.iter() {
        if waifu
            .name
            .to_lowercase()
            .starts_with(&partial.to_lowercase())
        {
            possible_options.push((waifu.name.clone(), waifu._id));
        }
    }

    let mut autocomplete_options = vec![];
    for (name, id) in possible_options {
        let ao = poise::AutocompleteChoice { name, value: id };
        autocomplete_options.push(ao);
    }

    autocomplete_options.into_iter()
}

/// Feed your waifus
#[poise::command(slash_command)]
pub async fn feed(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_waifu_name"]
    #[description = "Which waifu to feed"]
    waifu: u16,
    #[description = "What to feed your waifu"] food: FoodChoice,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let account = ctx.data().postgres.get_account(ctx.author().id).await?;
    let price = food.price();

    if account.currency < price as i32 {
        ctx.send(|cr| {
            cr.embed(|ce| {
                fmt::error(
                    "You don't have enough currency to feed your waifu this!",
                    ce,
                )
            })
        })
        .await?;
    } else {
        let waifu = ctx.data().mongo.get_waifu(waifu as i32).await?;
        let experience = food.random_experience();
        ctx.data()
            .postgres
            .update_experience(ctx.author().id, experience as i32)
            .await?;

        ctx.data()
            .postgres
            .update_currencies(ctx.author().id, -(price as i32), 0)
            .await?;

        ctx.send(|cr| {
            cr.embed(|ce| {
                fmt::success(
                    &format!(
                        "You fed {} some {} for {} :coin: - You gained {} experience",
                        &waifu.name,
                        food.icon(),
                        price,
                        experience
                    ),
                    ce,
                )
            })
        })
        .await?;
    }

    Ok(())
}

/// Sell a waifu for some currency
#[poise::command(slash_command)]
pub async fn sell(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_waifu_name"]
    #[description = "Which waifu to sell"]
    waifu: u16,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let waifu = ctx.data().mongo.get_waifu(waifu as i32).await?;
    let price = waifu.price();

    ctx.data()
        .postgres
        .remove_waifu(ctx.author().id, waifu._id)
        .await?;

    ctx.data()
        .postgres
        .update_currencies(ctx.author().id, price as i32, 0)
        .await?;

    ctx.send(|cr| {
        cr.embed(|ce| {
            fmt::success(
                &format!("Sold **`{}`** for **`{}`** :coin:", &waifu.name, price),
                ce,
            )
        })
    })
    .await?;

    Ok(())
}

pub fn commands() -> [crate::Command; 1] {
    [interact()]
}
