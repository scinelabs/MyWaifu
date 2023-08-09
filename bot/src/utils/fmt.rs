use poise::serenity_prelude::{Colour, CreateEmbed};

pub fn success<'a>(msg: &str, ce: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
    ce.colour(Colour::DARK_GREEN).description(msg)
}

pub fn error<'a>(msg: &str, ce: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
    ce.colour(Colour::RED).description(msg)
}
