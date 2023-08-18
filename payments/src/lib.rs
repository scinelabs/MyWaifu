mod fulfillments;
mod models;
mod stripe;

use worker::*;

use fulfillments::Fulfillments;
use models::CreatePaymentLink;
use stripe::{StripeClient, StripeEvent};

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    // Create an instance of the Router, which can use parameters (/user/:name) or wildcard values
    // (/file/*pathname). Alternatively, use `Router::with_data(D)` and pass in arbitrary data for
    // routes to access and share using the `ctx.data()` method.
    let router = Router::new();

    router
        .post_async("/cpl", create_payment_link)
        .post_async("/fulfill", fulfill_order)
        .run(req, env)
        .await
}

fn verify_request(ctx: &RouteContext<()>, req: &Request) -> bool {
    let authorization = req.headers().get("Authorization").unwrap();
    if let Some(auth) = authorization {
        let var = ctx.var("PAIR_SECRET").ok();
        if let Some(v) = var {
            let pair_secret = v.to_string();
            auth == pair_secret
        } else {
            false
        }
    } else {
        false
    }
}

pub async fn create_payment_link(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let request_ok = verify_request(&ctx, &req);
    if !request_ok {
        Response::error("Invalid Authorization", 403)
    } else {
        let data: CreatePaymentLink = req.json().await?;
        let data =
            StripeClient::create_payment_link(&ctx, &data.discord_user_id, &data.price_id).await?;
        Response::ok(data)
    }
}

pub async fn fulfill_order(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let stripe_event: StripeEvent = req.json().await?;

    if stripe_event.event_type == "checkout.session.completed" {
        let discord_id = stripe_event.data.object.metadata.discord_id;
        if let Some(user_id) = discord_id {
            Fulfillments::fulfill_order(&ctx, &user_id).await?;
            Response::ok("Fulfilled order")
        } else {
            Response::error("Missing 'discord_id' metadata", 400)
        }
    } else {
        Response::ok("Processed")
    }
}
