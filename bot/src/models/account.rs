#[derive(sqlx::FromRow)]
pub struct Account {
    pub user_id: i64,
    pub currency: i32,
    pub premium_currency: i32,
    pub waifus: Vec<i16>,
    pub packs: i16,
    pub premium_one_packs: i16,
    pub experience: i32,
}

#[derive(sqlx::FromRow)]
pub struct Alliance {
    pub owner: i64,
    pub name: String,
    pub members: Vec<i64>,
}
