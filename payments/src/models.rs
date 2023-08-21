use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreatePaymentLink {
    pub discord_user_id: String,
    pub price_id: String,
}

#[derive(Deserialize)]
pub struct ExchangePaymentCode {
    pub code: String,
}
