use poise::serenity_prelude as serenity;
use sqlx::{
    postgres::{PgPoolOptions, Postgres},
    Pool,
};

use crate::{
    config::Postgres as PostgresConfig,
    models::account::{Account, Alliance},
};

pub struct PostgresConnection {
    pool: Pool<Postgres>,
}
impl PostgresConnection {
    pub async fn connect(config: &PostgresConfig) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect(&config.connection_uri)
            .await
            .expect("Failed to connect to POSTGRES database");

        Self { pool }
    }
    pub async fn register_account(&self, user_id: serenity::UserId) -> Result<(), crate::Error> {
        sqlx::query("INSERT INTO accounts VALUES($1)")
            .bind(user_id.0 as i64)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
    pub async fn get_account(&self, user_id: serenity::UserId) -> Result<Account, crate::Error> {
        let account = sqlx::query_as("SELECT * FROM accounts WHERE user_id = $1")
            .bind(user_id.0 as i64)
            .fetch_one(&self.pool)
            .await?;

        Ok(account)
    }
    pub async fn delete_account(&self, user_id: serenity::UserId) -> Result<(), crate::Error> {
        sqlx::query("DELETE FROM accounts WHERE user_id = $1")
            .bind(user_id.0 as i64)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
    pub async fn update_currencies(
        &self,
        user_id: serenity::UserId,
        currency: i32,
        premium_currency: i32,
    ) -> Result<(), crate::Error> {
        sqlx::query(
            "UPDATE accounts SET currency = currency + $1, premium_currency = premium_currency + $1 WHERE user_id = $3"
        ).bind(currency).bind(premium_currency).bind(user_id.0 as i64).execute(&self.pool).await?;

        Ok(())
    }
    pub async fn update_experience(
        &self,
        user_id: serenity::UserId,
        amount: i32,
    ) -> Result<(), crate::Error> {
        sqlx::query("UPDATE accounts SET experience = experience + $1 WHERE user_id = $2")
            .bind(amount)
            .bind(user_id.0 as i64)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
    pub async fn update_packs(
        &self,
        user_id: serenity::UserId,
        packs: i16,
        premium_one_packs: i16,
    ) -> Result<(), crate::Error> {
        sqlx::query(
            "UPDATE accounts SET packs = packs + $1, premium_one_packs = premium_one_packs + $2 WHERE user_id = $3"
        ).bind(packs).bind(premium_one_packs).bind(user_id.0 as i64).execute(&self.pool).await?;

        Ok(())
    }
    pub async fn add_waifu(
        &self,
        user_id: serenity::UserId,
        waifu_id: u16,
    ) -> Result<(), crate::Error> {
        sqlx::query("UPDATE accounts SET waifus = array_append(waifus, $1) WHERE user_id = $2")
            .bind(waifu_id as i16)
            .bind(user_id.0 as i64)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
    pub async fn remove_waifu(
        &self,
        user_id: serenity::UserId,
        waifu_id: u16,
    ) -> Result<(), crate::Error> {
        sqlx::query("UPDATE accounts SET waifus = array_remove(waifus, $1) WHERE user_id = $2")
            .bind(waifu_id as i16)
            .bind(user_id.0 as i64)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
    pub async fn get_waifus(&self, user_id: serenity::UserId) -> Result<Vec<i16>, crate::Error> {
        let (waifus,) = sqlx::query_as("SELECT waifus FROM accounts WHERE user_id = $1")
            .bind(user_id.0 as i64)
            .fetch_one(&self.pool)
            .await?;

        Ok(waifus)
    }
    pub async fn get_alliance(&self, user_id: serenity::UserId) -> Result<Alliance, crate::Error> {
        let alliance =
            sqlx::query_as("SELECT * FROM alliances WHERE owner = $1 OR $1 = ANY(members)")
                .bind(user_id.0 as i64)
                .fetch_one(&self.pool)
                .await?;

        Ok(alliance)
    }
    pub async fn create_alliance(
        &self,
        user_id: serenity::UserId,
        name: &str,
    ) -> Result<(), crate::Error> {
        sqlx::query("INSERT INTO alliances VALUES($1, $2)")
            .bind(user_id.0 as i64)
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
