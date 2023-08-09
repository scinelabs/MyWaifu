use futures::StreamExt;
use poise::serenity_prelude as serenity;

use crate::utils::random_component_id;

pub struct ConfirmMenu;
impl ConfirmMenu {
    pub async fn start(ctx: crate::Context<'_>, text: &str) -> Result<bool, crate::Error> {
        let (confirm_id, cancel_id) = (random_component_id(), random_component_id());

        let handle = ctx
            .send(|cr| {
                cr.embed(|ce| {
                    ce.title("Woah! Hold up!")
                        .colour(serenity::Colour::ORANGE)
                        .description(text)
                })
                .components(|cc| {
                    cc.create_action_row(|car| {
                        car.create_button(|cb| {
                            cb.label("Confirm")
                                .style(serenity::ButtonStyle::Success)
                                .custom_id(&confirm_id)
                        })
                        .create_button(|cb| {
                            cb.label("Cancel")
                                .style(serenity::ButtonStyle::Danger)
                                .custom_id(&cancel_id)
                        })
                    })
                })
            })
            .await?;

        let message = handle.message().await?;
        let mut collector = message
            .await_component_interactions(&ctx.serenity_context().shard)
            .timeout(std::time::Duration::from_secs(60 * 2))
            .build();

        while let Some(interaction) = collector.next().await {
            return Ok(&interaction.data.custom_id == &confirm_id);
        }

        return Ok(false);
    }
}
