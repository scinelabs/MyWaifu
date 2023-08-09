use crate::{
    components::{
        choice::ChoicePrompt,
        shop::{Item, Shop},
    },
    utils::fmt,
    Context, Error,
};

#[poise::command(
    slash_command,
    subcommands("packs"),
    check = "crate::checks::has_account"
)]
pub async fn shop(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Shop for some packs
#[poise::command(slash_command)]
pub async fn packs(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let account = ctx.data().postgres.get_account(ctx.author().id).await?;

    let items = vec![
        Item::new(
            "Standard Pack",
            "Comes with 3 waifus (any rarity)",
            (250, Some(50)),
        ),
        Item::new(
            "Gold Pack",
            "Comes with 5 waifus (any rarity)",
            (475, Some(100)),
        ),
    ];
    let shop = Shop::new(
        "Pack Shop",
        "Purchase some packs with currency or premium currency. You can purchase waifus directly with premium currency.",
        items,
    );
    let chosen_item = shop.start(ctx).await?;
    if let Some(chosen_item) = chosen_item {
        let choice_menu = ChoicePrompt::new(vec![("Standard", 1), ("Premium", 2)]);
        let chosen_choice = choice_menu
            .start(
                ctx,
                "Currency use",
                "What type of currency do you want to use?",
            )
            .await?;
        if let Some(chosen_choice) = chosen_choice {
            let mut currency: i32 = 0;
            let mut premium_currency: i32 = 0;
            if chosen_choice == 1 {
                if account.currency < chosen_item.price.0 {
                    ctx.send(|cr| {
                        cr.embed(|ce| fmt::error("You don't have enough to purchase this item", ce))
                    })
                    .await?;
                    return Ok(());
                }
                currency = chosen_item.price.0;
            } else {
                premium_currency = chosen_item.price.1.unwrap();
                if account.premium_currency < premium_currency {
                    ctx.send(|cr| {
                        cr.embed(|ce| fmt::error("You don't have enough to purchase this item", ce))
                    })
                    .await?;
                    return Ok(());
                }
            }

            ctx.data()
                .postgres
                .update_currencies(ctx.author().id, -currency, -premium_currency)
                .await?;

            if chosen_item.name.as_str() == "Standard Pack" {
                ctx.data()
                    .postgres
                    .update_packs(ctx.author().id, 1, 0)
                    .await?;
            } else {
                ctx.data()
                    .postgres
                    .update_packs(ctx.author().id, 0, 1)
                    .await?;
            }

            ctx.send(|cr| {
                cr.embed(|ce| {
                    fmt::success(
                        "Purchase successfully completed. Validate by running `/account view`",
                        ce,
                    )
                })
                .ephemeral(true)
            })
            .await?;
        }
    }

    Ok(())
}

pub fn commands() -> [crate::Command; 1] {
    [shop()]
}
