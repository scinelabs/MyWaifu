use serde_json::json;
use twilight_util::builder::embed::{EmbedAuthorBuilder, EmbedBuilder, ImageSource};
use worker::{Fetch, Headers, Method, Request, RequestInit, Result, RouteContext};

const BASE: &str = "https://discord.com/api/v10";

pub struct Fulfillments;
impl Fulfillments {
    pub async fn fulfill_order(ctx: &RouteContext<()>, user_id: &str) -> Result<()> {
        Self::send_discord_message(ctx, user_id).await.ok();

        Ok(())
    }
    async fn send_discord_message(ctx: &RouteContext<()>, user_id: &str) -> Result<()> {
        let embed = EmbedBuilder::new()
            .title("Order Complete")
            .description(
                "Hey there. Thanks for your purchase, your contribution will go towards paying for our server costs."
            ).image(ImageSource::url("https://media.discordapp.net/attachments/1140568264456544348/1141347789884887070/Powered_by_Stripe_-_blurple-600x136-06cf2b6.png?width=1200&height=272").unwrap())
            .author(EmbedAuthorBuilder::new("Scine Labs").icon_url(ImageSource::url("https://media.discordapp.net/attachments/1140568264456544348/1142052574695010325/121638661.png?width=400&height=400").unwrap()))
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
                Fetch::Request(create_message_request).send().await.ok(); // doesn't matter if this goes through
            }
        }

        Ok(())
    }
}
