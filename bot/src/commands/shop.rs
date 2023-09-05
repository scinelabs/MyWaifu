use poise::serenity_prelude::{ButtonStyle, CacheHttp};

use crate::{
    components::{
        choice::ChoicePrompt,
        shop::{Item, Shop},
    },
    utils::fmt,
    Context, Error,
};

const SCINE_INVITE: &str = "https://discord.gg/2RTEu23AZP";

#[poise::command(
    slash_command,
    subcommands("packs", "premium", "exchange"),
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

#[derive(serde::Deserialize)]
pub struct UrlKey {
    pub url: String,
}

#[derive(Debug, poise::ChoiceParameter)]
pub enum CrateOption {
    #[name = "Standard Crate - 2,500 🪙 + 150 🍪 + 15 Packs"]
    Standard,
    #[name = "Diamond Crate - 5,000 🪙 + 300 🍪 + 30 Packs"]
    Diamond,
}
impl CrateOption {
    pub fn price_id(&self) -> &str {
        match self {
            Self::Standard => "standard_crate_mywaifu",
            Self::Diamond => "diamond_crate_mywaifu",
        }
    }
}

/// Shop for items with real money
#[poise::command(slash_command)]
pub async fn premium(
    ctx: Context<'_>,
    #[description = "What crate you want to purchase"] choice: CrateOption,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let stripe_url = ctx.data().conf.stripe.format_stripe_hook_url("/cpl");
    let body = serde_json::json!({
        "discord_user_id": ctx.author().id.0.to_string(),
        "price_id": choice.price_id()
    })
    .to_string();

    let resp = ctx
        .data()
        .http
        .post(stripe_url)
        .body(body)
        .header("Authorization", &ctx.data().conf.stripe.cloudflare_auth)
        .send()
        .await?;
    let data: UrlKey = resp.json().await?;

    let dm_result = ctx.author()
        .direct_message(ctx.http(), |cm| {
            cm.embed(|ce| {
                ce.title("Crate Purchase").description(
                    ":warning: Please have your DMs open until the purchase has been completed.\nYou will receive a DM after the purchase has been completed.\n\nPlease join the support server if you have any issues.",
                ).field("Purchase URL", &data.url, false).author(|ca| ca.icon_url("https://media.discordapp.net/attachments/1140568264456544348/1142052574695010325/121638661.png?width=400&height=400").name("Scine Labs"))
            }).components(|cc| cc.create_action_row(|car| car.create_button(|cb| cb.label("Scine Labs").style(ButtonStyle::Link).url(SCINE_INVITE))))
        })
        .await;

    if dm_result.is_ok() {
        ctx.send(|cr| cr.embed(|ce| fmt::success("Please check your DMs. **Note:** Your DMs must remain open till the purchase has been completed.", ce)))
            .await?;
    } else {
        ctx.send(|cr| {
            cr.embed(|ce| {
                fmt::error(
                    "Please open DMs. Your DMs must be open till the purchase has been completed.",
                    ce,
                )
            })
        })
        .await?;
    }

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct Metadata {
    pub discord_id: String,
    pub price_id: String,
}

/// Exchange a payment code for your items
#[poise::command(slash_command)]
pub async fn exchange(ctx: Context<'_>, code: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let worker_url = ctx.data().conf.stripe.format_stripe_hook_url("/epc");

    let payload = serde_json::json!({
        "code": code
    })
    .to_string();
    let resp = ctx
        .data()
        .http
        .post(worker_url)
        .body(payload)
        .header("Authorization", &ctx.data().conf.stripe.cloudflare_auth)
        .send()
        .await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        ctx.send(|cr| cr.embed(|ce| fmt::error("Invalid payment code.", ce)))
            .await?;
    } else if !resp.status().is_success() {
        ctx.send(|cr| cr.embed(|ce| fmt::error("An unknown error occurred. Try again later.", ce)))
            .await?;
    } else {
        // all good
        let data: Metadata = resp.json().await?;
        let product = ctx
            .data()
            .postgres
            .get_premium_product(&data.price_id)
            .await?;

        ctx.data()
            .postgres
            .update_packs(ctx.author().id, product.packs, product.premium_one_packs)
            .await?;
        ctx.data()
            .postgres
            .update_currencies(ctx.author().id, product.currency, product.premium_currency)
            .await?;

        ctx.send(|cr| {
            cr.embed(|ce| fmt::success("Successfully redeemed items. Enjoy and thank you!", ce))
        })
        .await?;
    }

    Ok(())
}

pub fn commands() -> [crate::Command; 1] {
    [shop()]
}
