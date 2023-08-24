use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::{Fetch, Headers, Method, Request, RequestInit, Result, RouteContext};

const VERSION: u8 = 1;

pub struct StripeClient;
impl StripeClient {
    fn create_url(key: &str, path: &str, body: serde_json::Value) -> String {
        let url_form = serde_urlencoded::to_string(body).expect("Failed to urlencode body");
        format!("https://{key}:@api.stripe.com/v{VERSION}{path}?{url_form}")
    }
    fn extract_stripe_key(ctx: &RouteContext<()>) -> String {
        let var = ctx.var("STRIPE_KEY").expect("No STRIPE_KEY found");
        let stripe_key = var.to_string();
        stripe_key
    }
    pub async fn create_payment_link(
        ctx: &RouteContext<()>,
        discord_id: &str,
        price_id: &str,
    ) -> Result<String> {
        let key = Self::extract_stripe_key(ctx);
        let data = json!({
            "line_items[0][price]": price_id,
            "line_items[0][quantity]": 1,
            "metadata[discord_id]": discord_id,
            "metadata[price_id]": price_id
        });
        let url = Self::create_url(&key, "/payment_links", data);

        let mut init = RequestInit::default();
        init.with_method(Method::Post);

        let mut headers = Headers::new();
        headers.append("Authorization", &format!("Bearer {key}"))?;
        headers.append("Content-Type", "application/x-www-form-urlencoded")?;
        init.with_headers(headers);

        let request = Request::new_with_init(&url, &init)?;

        let mut response = Fetch::Request(request).send().await?;

        if response.status_code() > 400 {
            Err(worker::Error::Internal("Request failed to stripe".into()))
        } else {
            let text = response.text().await?;
            Ok(text)
        }
    }
}

#[derive(Deserialize)]
pub struct StripeEvent {
    pub id: String,
    pub data: StripeEventObjectParent,
    #[serde(rename = "type")]
    pub event_type: String,
}

#[derive(Deserialize)]
pub struct StripeEventObjectParent {
    pub object: StripeEventObject,
}

#[derive(Deserialize)]
pub struct StripeEventObject {
    pub id: String,
    pub metadata: StripeEventMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StripeEventMetadata {
    pub discord_id: Option<String>, // not all stripe events will contain metadata
    pub price_id: Option<String>,
}

// https://github.com/arlyon/async-stripe/blob/master/src/resources/webhook_events.rs#L537
#[derive(Debug)]
pub struct Signature<'r> {
    pub t: i64,
    pub v1: &'r str,
}

impl<'r> Signature<'r> {
    pub fn parse(raw: &'r str) -> Result<Signature<'r>> {
        let headers: HashMap<&str, &str> = raw
            .split(',')
            .map(|header| {
                let mut key_and_value = header.split('=');
                let key = key_and_value.next();
                let value = key_and_value.next();
                (key, value)
            })
            .filter_map(|(key, value)| match (key, value) {
                (Some(key), Some(value)) => Some((key, value)),
                _ => None,
            })
            .collect();
        let t = headers
            .get("t")
            .ok_or(worker::Error::RustError("Bad Signature".into()))?;
        let v1 = headers
            .get("v1")
            .ok_or(worker::Error::RustError("Bad Signature".into()))?;
        Ok(Signature {
            t: t.parse::<i64>()
                .map_err(|_| worker::Error::RustError("Bad Header".into()))?,
            v1,
        })
    }
}
