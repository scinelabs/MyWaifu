mod checks;
mod commands;
mod components;
mod config;
mod database;
mod models;
mod utils;

use poise::serenity_prelude::{self as serenity, GuildId};

use checks::CheckCache;
use database::{mongo::MongoConnection, postgres::PostgresConnection};

pub struct Data {
    postgres: PostgresConnection,
    mongo: MongoConnection,
    check_cache: CheckCache,
} // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type Command = poise::Command<Data, Error>;

#[tokio::main]
async fn main() {
    let cli_args: Vec<String> = std::env::args().collect();
    let should_resync = cli_args.contains(&String::from("--resync"));

    let conf = config::Config::read();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::commands(),
            ..Default::default()
        })
        .token(&conf.discord.token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                if should_resync {
                    println!("Resyncing commands (global and test guild)!");
                    let guild_id = GuildId::from(conf.discord.test_guild_id);
                    poise::builtins::register_in_guild(
                        &ctx.http,
                        &framework.options().commands,
                        guild_id,
                    )
                    .await?;

                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }

                let postgres_connection = PostgresConnection::connect(&conf.postgres).await;
                let mongo_connection = MongoConnection::connect(&conf.mongo).await;
                Ok(Data {
                    postgres: postgres_connection,
                    mongo: mongo_connection,
                    check_cache: CheckCache::new(),
                })
            })
        });

    println!("Running bot");
    framework.run().await.expect("Framework cannot run the bot");
}
