use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

use crate::utils::ToEmbed;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Waifu {
    pub _id: u16,
    pub name: String,
    pub description: String,
    pub gdrive_id: String,
    pub likes: u32,
    pub trash: u32,
}
impl Waifu {
    pub fn download_url(&self) -> String {
        format!(
            "https://worker-rust.reachvishm8605.workers.dev/image/{}",
            self.gdrive_id
        )
    }
    pub fn price(&self) -> u32 {
        let calculated = (self.likes - self.trash) + 175;
        if calculated <= 0 {
            0
        } else {
            calculated
        }
    }
}
impl ToEmbed for Waifu {
    fn to_embed<'a>(&self, ce: &'a mut serenity::CreateEmbed) -> &'a mut serenity::CreateEmbed {
        ce.image(&self.download_url())
            .colour(serenity::Colour::FABLED_PINK)
            .title(&self.name)
            .description(&self.description)
    }
}
