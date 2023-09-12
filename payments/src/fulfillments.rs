use serde_json::json;
use twilight_util::builder::embed::{
    EmbedAuthorBuilder, EmbedBuilder, EmbedFieldBuilder, ImageSource,
};
use worker::*;

use crate::stripe::StripeEventMetadata;

const BASE: &str = "https://discord.com/api/v10";

pub struct Fulfillments;
impl Fulfillments {
    pub async fn fulfill_order(
        ctx: &RouteContext<()>,
        metadata: StripeEventMetadata,
    ) -> Result<()> {
        let code = Self::generate_code(ctx, &metadata).await?;
        Self::send_discord_message(ctx, &metadata.discord_id.unwrap(), &code).await?;

        Ok(())
    }
    async fn generate_code(
        ctx: &RouteContext<()>,
        metadata: &StripeEventMetadata,
    ) -> Result<String> {
        let uid = uuid::Uuid::new_v4();
        let payment_code = uid.simple().to_string();

        let kv = ctx.kv("PAYMENT_CODES")?;
        kv.put(&payment_code, serde_json::to_string(&metadata).unwrap())?
            .expiration_ttl(86400 * 3) // 3 days
            .execute()
            .await?;

        Ok(payment_code)
    }
    async fn send_discord_message(ctx: &RouteContext<()>, user_id: &str, code: &str) -> Result<()> {
        let code_fmt_msg = format!("`{code}`\n\nRun `/shop exchange {code}` to get your items.");
        let embed = EmbedBuilder::new()
            .title("Order Complete")
            .description(
                "Hey there. Thanks for your purchase, your contribution will go towards paying for our server costs."
            ).image(ImageSource::url("https://media.discordapp.net/attachments/1140568264456544348/1141347789884887070/Powered_by_Stripe_-_blurple-600x136-06cf2b6.png?width=1200&height=272").unwrap())
            .author(EmbedAuthorBuilder::new("Scine Labs").icon_url(ImageSource::url("https://media.discordapp.net/attachments/1140568264456544348/1142052574695010325/121638661.png?width=400&height=400").unwrap()))
            .field(EmbedFieldBuilder::new("Payment Code", code_fmt_msg))
            .build();

        let bot_token = ctx
            .var("BOT_TOKEN")
            .expect("Did not find env var")
            .to_string();

        let mut headers = Headers::new();
        headers
            .append("Authorization", &format!("Bot {bot_token}"))
            .expect("Invalid Auth headers");
        headers
            .append("Content-Type", "application/json")
            .expect("Invalid content-type headers");

        let create_dm_url = format!("{BASE}/users/@me/channels");
        let create_dm_body = json!({
            "recipient_id": user_id
        })
        .to_string();

        let mut create_dm_init = RequestInit::default();
        create_dm_init.with_method(Method::Post);
        create_dm_init.with_body(Some(create_dm_body.into()));
        create_dm_init.with_headers(headers.clone());

        let create_dm_request = Request::new_with_init(&create_dm_url, &create_dm_init)?;
        let response = Fetch::Request(create_dm_request).send().await;
        if let Ok(mut resp) = response {
            let data_result: Result<twilight_model::channel::Channel> = resp.json().await;
            if let Ok(channel) = data_result {
                let channel_id = channel.id.get();
                let create_message_url = format!("{BASE}/channels/{channel_id}/messages");
                let create_message_body = json!({
                    "embeds": [embed]
                })
                .to_string();
                let mut create_message_init = RequestInit::new();
                create_message_init.with_body(Some(create_message_body.into()));
                create_message_init.with_headers(headers);
                create_message_init.with_method(Method::Post);

                let create_message_request =
                    Request::new_with_init(&create_message_url, &create_message_init).unwrap();
                Fetch::Request(create_message_request).send().await?;
            }
        }

        Ok(())
    }
}
