use std::collections::HashMap;

use futures::StreamExt;
use poise::serenity_prelude as serenity;

use crate::utils::{random_component_id, ToEmbed};

#[derive(Debug)]
pub struct Item {
    pub name: String,
    pub description: String,
    pub price: (i32, Option<i32>),
}
impl Item {
    pub fn new(name: &str, description: &str, price: (i32, Option<i32>)) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            price,
        }
    }
}

pub struct Shop {
    items: Vec<Item>,
    name: String,
    description: String,
}
impl Shop {
    pub fn new(name: &str, description: &str, items: Vec<Item>) -> Self {
        Self {
            items,
            name: name.into(),
            description: description.into(),
        }
    }
    pub async fn start(&self, ctx: crate::Context<'_>) -> Result<Option<&Item>, crate::Error> {
        let mut component_ids = HashMap::new();
        for item in self.items.iter() {
            let component_id = random_component_id();
            component_ids.insert(component_id, item);
        }

        let handle = ctx
            .send(|cr| {
                cr.embed(|ce| self.to_embed(ce)).components(|cc| {
                    cc.create_action_row(|car| {
                        for (c_id, item) in component_ids.iter() {
                            car.create_button(|cb| cb.label(&item.name).custom_id(&c_id));
                        }
                        car
                    })
                })
            })
            .await?;
        let message = handle.message().await?;
        let mut collector = message
            .await_component_interactions(&ctx.serenity_context().shard)
            .timeout(std::time::Duration::from_secs(60 * 3))
            .build();

        while let Some(interaction) = collector.next().await {
            let interaction_id = interaction.data.custom_id.clone();
            let item = component_ids.get(&interaction_id).unwrap();
            return Ok(Some(*item));
        }

        Ok(None)
    }
    fn format_price(&self, price: (i32, Option<i32>)) -> String {
        let (currency, premium_currency) = price;
        if let Some(pc) = premium_currency {
            format!(":coin: {}\n:cookie: {}", currency, pc)
        } else {
            format!(":coin: {}", currency)
        }
    }
}

impl ToEmbed for Shop {
    fn to_embed<'a>(&self, ce: &'a mut serenity::CreateEmbed) -> &'a mut serenity::CreateEmbed {
        ce
            .title(&self.name)
            .description(format!("{} \n\nClick on the buttons below to purchase something. You will **not** receive a confirmation prompt.", self.description))
            .image("https://media.discordapp.net/attachments/1135599535847120896/1135864814149828638/a2951652cffcba42fe8b6d010d9e5dd0.gif?width=1440&height=920");
        for item in self.items.iter() {
            ce.field(&item.name, self.format_price(item.price), true);
        }
        ce
    }
}
