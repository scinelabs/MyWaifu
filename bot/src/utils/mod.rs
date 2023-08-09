pub mod fmt;

use poise::serenity_prelude as serenity;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

pub fn random_component_id() -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    rand_string
}

pub trait ToEmbed {
    fn to_embed<'a>(&self, ce: &'a mut serenity::CreateEmbed) -> &'a mut serenity::CreateEmbed;
}
