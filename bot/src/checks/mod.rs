mod accounts;
mod alliances;

use std::collections::HashMap;

use poise::serenity_prelude as serenity;
use tokio::sync::Mutex as TokioMutex;

pub use accounts::has_account;
pub use alliances::in_alliance;

pub struct CheckCache {
    has_account_cache: TokioMutex<HashMap<u64, bool>>,
    in_alliance_cache: TokioMutex<HashMap<u64, bool>>,
}
impl CheckCache {
    pub fn new() -> Self {
        Self {
            has_account_cache: TokioMutex::new(HashMap::new()),
            in_alliance_cache: TokioMutex::new(HashMap::new()),
        }
    }
    pub async fn insert_has_account(&self, user_id: serenity::UserId, value: bool) {
        let mut guard = self.has_account_cache.lock().await;
        guard.insert(user_id.0, value);
    }
    pub async fn get_has_account(&self, user_id: serenity::UserId) -> Option<bool> {
        let guard = self.has_account_cache.lock().await;
        let value = guard.get(&user_id.0);
        match value {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }

    pub async fn insert_in_alliance(&self, user_id: serenity::UserId, value: bool) {
        let mut guard = self.in_alliance_cache.lock().await;
        guard.insert(user_id.0, value);
    }
    pub async fn get_in_alliance(&self, user_id: serenity::UserId) -> Option<bool> {
        let guard = self.in_alliance_cache.lock().await;
        let value = guard.get(&user_id.0);
        match value {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }
}
