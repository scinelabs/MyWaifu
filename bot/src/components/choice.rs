use std::collections::HashMap;

use futures::StreamExt;

use crate::utils::random_component_id;

pub struct ChoicePrompt {
    choices: Vec<(String, u8)>,
}
impl ChoicePrompt {
    pub fn new(choices: Vec<(&str, u8)>) -> Self {
        let transformed = choices
            .iter()
            .map(|(label, id)| (label.to_string(), id.clone()))
            .collect();

        Self {
            choices: transformed,
        }
    }
    pub async fn start(
        &self,
        ctx: crate::Context<'_>,
        title: &str,
        description: &str,
    ) -> Result<Option<u8>, crate::Error> {
        let mut component_ids = HashMap::new();
        for (label, id) in self.choices.iter() {
            component_ids.insert(random_component_id(), (label.clone(), id.clone()));
        }
        let handle = ctx
            .send(|cr| {
                cr.embed(|ce| ce.title(title).description(description))
                    .components(|cc| {
                        cc.create_action_row(|car| {
                            for (c_id, (label, _)) in component_ids.iter() {
                                car.create_button(|cb| cb.label(label).custom_id(c_id));
                            }
                            car
                        })
                    })
                    .ephemeral(true)
            })
            .await?;
        let message = handle.message().await?;
        let mut collector = message
            .await_component_interactions(&ctx.serenity_context().shard)
            .timeout(std::time::Duration::from_secs(60 * 3))
            .author_id(ctx.author().id)
            .build();

        while let Some(interaction) = collector.next().await {
            let (_, id) = component_ids.get(&interaction.data.custom_id).unwrap();
            return Ok(Some(id.clone()));
        }

        Ok(None)
    }
}
