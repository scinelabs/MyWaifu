use std::collections::VecDeque;

use futures::StreamExt;
use poise::serenity_prelude::{ButtonStyle, CacheHttp};

use crate::utils::{random_component_id, ToEmbed};

pub struct EmbedPaginator<T: ToEmbed> {
    items: VecDeque<T>,
}
impl<T: ToEmbed> EmbedPaginator<T> {
    pub fn new(items: Vec<T>) -> Self {
        let items = VecDeque::from(items);
        Self { items }
    }
    pub async fn start(
        &mut self,
        ctx: crate::Context<'_>,
        with_select: bool,
    ) -> Result<Option<&T>, crate::Error> {
        let first = self.items.get(0).expect("Must have at least 1 item");

        let (left_id, right_id, select_id) = (
            random_component_id(),
            random_component_id(),
            random_component_id(),
        );

        let handle = ctx
            .send(|cr| {
                cr.embed(|ce| first.to_embed(ce)).components(|cc| {
                    cc.create_action_row(|car| {
                        car.create_button(|cb| cb.label("<-").custom_id(left_id.clone()))
                            .create_button(|cb| cb.label("->").custom_id(right_id));

                        if with_select {
                            car.create_button(|cb| {
                                cb.label("Select")
                                    .custom_id(select_id.clone())
                                    .style(ButtonStyle::Success)
                            });
                        };

                        car
                    })
                })
            })
            .await?;

        let message = handle.message().await?;
        let mut collector = message
            .await_component_interactions(&ctx.serenity_context().shard)
            .timeout(std::time::Duration::from_secs(60 * 5))
            .author_id(ctx.author().id)
            .build();

        while let Some(interaction) = collector.next().await {
            if &interaction.data.custom_id == &left_id {
                self.items.rotate_left(1);
            } else if &interaction.data.custom_id == &select_id {
                // the user clicked the "Select" button
                // indicating they're selecting the currently rendered item
                let item = self.items.get(0).unwrap();
                return Ok(Some(item));
            } else {
                self.items.rotate_right(1);
            }

            let item = self.items.get(0).unwrap();
            handle
                .edit(ctx, |cr| cr.embed(|ce| item.to_embed(ce)))
                .await?;
            interaction.defer(ctx.http()).await?;
        }

        Ok(None)
    }
}
