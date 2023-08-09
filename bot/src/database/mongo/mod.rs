use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Bson},
    options::{ClientOptions, ResolverConfig},
    Client, Collection,
};

use crate::{config::Mongo, models::waifu::Waifu};

pub struct MongoConnection {
    waifu_collection: Collection<Waifu>,
}
impl MongoConnection {
    pub async fn connect(config: &Mongo) -> Self {
        let client_options = ClientOptions::parse_with_resolver_config(
            &config.connection_uri,
            ResolverConfig::cloudflare(),
        )
        .await
        .expect("Cannot parse mongodb connection uri");

        let client = Client::with_options(client_options)
            .expect("Failed to build client with client options");

        let core_db = client.database("core");
        let waifu_collection = core_db.collection::<Waifu>("waifus");

        Self { waifu_collection }
    }
    pub async fn get_random_waifus(
        &self,
        count: u32,
        current_waifus: &Vec<i16>,
    ) -> Result<Vec<Waifu>, crate::Error> {
        let current_waifus: Vec<i32> = current_waifus.iter().map(|el| el.clone() as i32).collect();
        let query = doc! { "$sample": { "size": count } };
        let query2 = doc! { "$match": { "_id": { "$nin": current_waifus } } };
        let mut cursor = self
            .waifu_collection
            .aggregate([query2, query], None)
            .await?;
        let mut documents = vec![];
        while let Some(doc) = cursor.try_next().await? {
            let waifu: Waifu = mongodb::bson::from_bson(Bson::Document(doc))?;
            documents.push(waifu)
        }

        Ok(documents)
    }
    pub async fn get_waifus(&self, waifu_ids: Vec<i32>) -> Result<Vec<Waifu>, crate::Error> {
        let query = doc! { "_id": { "$in": waifu_ids } };
        let mut cursor = self.waifu_collection.find(query, None).await?;
        let mut documents = vec![];
        while let Some(doc) = cursor.try_next().await? {
            documents.push(doc);
        }

        Ok(documents)
    }
    pub async fn get_waifu(&self, waifu_id: i32) -> Result<Waifu, crate::Error> {
        let query = doc! { "_id": waifu_id };
        let waifu = self.waifu_collection.find_one(query, None).await?;

        Ok(waifu.unwrap())
    }
    pub async fn search_waifus(&self, query: &str) -> Result<Vec<Waifu>, crate::Error> {
        let mongo_query = doc! { "name": { "$regex": query, "$options": "i" } };
        let mut cursor = self.waifu_collection.find(mongo_query, None).await?;
        let mut documents = vec![];
        while let Some(doc) = cursor.try_next().await? {
            documents.push(doc);
        }

        Ok(documents)
    }
}
