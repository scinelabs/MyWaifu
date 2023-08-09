use crate::{utils::fmt, Context, Error};

pub async fn has_account(ctx: Context<'_>) -> Result<bool, Error> {
    let cache_option = ctx
        .data()
        .check_cache
        .get_has_account(ctx.author().id)
        .await;
    if let Some(has_account_value) = cache_option {
        // value was found in the cache
        // so we'll just return it
        Ok(has_account_value)
    } else {
        let postgres_result = ctx.data().postgres.get_account(ctx.author().id).await;
        ctx.data()
            .check_cache
            .insert_has_account(ctx.author().id, postgres_result.is_ok())
            .await;

        if postgres_result.is_err() {
            ctx.send(|cr| {
                cr.ephemeral(true).embed(|ce| fmt::error("You must have an account to run this command. If you believe this is an error, contact our support team.", ce))
            })
            .await
            .ok();
        }

        Ok(postgres_result.is_ok())
    }
}
