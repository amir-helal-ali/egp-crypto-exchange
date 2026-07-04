//! Redis-backed queue + pub/sub for manual transaction status updates.
//!
//! Three concerns:
//! 1. **Pending queue** — `LPUSH`/`RPOP` (FIFO) list per (type, asset_class).
//!    Workers (admin actions) atomically pop and re-insert on review.
//! 2. **Status broadcast** — Pub/Sub channel `manual_tx:<id>` publishes status
//!    changes so connected users see real-time updates.
//! 3. **Cached position** — `HSET pending:<id>` stores position-in-queue count
//!    so users can see "you are #3 in queue".

use std::sync::Arc;
use std::time::Duration;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::ManualTxStatus;

pub struct RedisQueue {
    pub conn: ConnectionManager,
    pub prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEvent {
    pub manual_tx_id: Uuid,
    pub user_id: Uuid,
    pub status: ManualTxStatus,
    pub queue_position: u32,
    pub ts: chrono::DateTime<chrono::Utc>,
}

impl RedisQueue {
    pub async fn connect(url: &str, prefix: &str) -> AppResult<Self> {
        let client = redis::Client::open(url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self {
            conn,
            prefix: prefix.to_string(),
        })
    }

    // --- key helpers ---------------------------------------------------------
    fn key_pending_list(&self, tx_type: &str, asset_class: &str) -> String {
        format!("{}:pending:{}:{}", self.prefix, tx_type, asset_class)
    }
    fn key_status_channel(&self, user_id: Uuid) -> String {
        format!("{}:status:user:{}", self.prefix, user_id)
    }
    fn key_user_queue(&self, user_id: Uuid) -> String {
        format!("{}:user_queue:{}", self.prefix, user_id)
    }

    // --- enqueue / dequeue ---------------------------------------------------

    /// Enqueue a manual transaction id into the appropriate FIFO list.
    pub async fn enqueue(
        &self,
        manual_tx_id: Uuid,
        user_id: Uuid,
        tx_type: &str,
        asset_class: &str,
    ) -> AppResult<()> {
        let mut conn = self.conn.clone();
        let list_key = self.key_pending_list(tx_type, asset_class);
        let payload = serde_json::to_string(&(manual_tx_id, user_id))?;
        let _: () = conn.rpush(&list_key, payload).await?;
        Ok(())
    }

    /// Snapshot of pending ids (admin view). Returns up to `limit` items.
    pub async fn pending_list(
        &self,
        tx_type: &str,
        asset_class: &str,
        limit: i64,
    ) -> AppResult<Vec<(Uuid, Uuid)>> {
        let mut conn = self.conn.clone();
        let list_key = self.key_pending_list(tx_type, asset_class);
        let raw: Vec<String> = conn.lrange(&list_key, 0, limit.saturating_sub(1)).await?;
        let mut out = Vec::with_capacity(raw.len());
        for r in raw {
            if let Ok((id, uid)) = serde_json::from_str::<(Uuid, Uuid)>(&r) {
                out.push((id, uid));
            }
        }
        Ok(out)
    }

    /// Compute the position of a specific manual_tx_id inside its queue.
    pub async fn position(
        &self,
        manual_tx_id: Uuid,
        tx_type: &str,
        asset_class: &str,
    ) -> AppResult<u32> {
        let mut conn = self.conn.clone();
        let list_key = self.key_pending_list(tx_type, asset_class);
        let raw: Vec<String> = conn.lrange(&list_key, 0, -1).await?;
        for (i, r) in raw.iter().enumerate() {
            if let Ok((id, _)) = serde_json::from_str::<(Uuid, Uuid)>(r) {
                if id == manual_tx_id {
                    return Ok(i as u32 + 1);
                }
            }
        }
        Ok(0) // not in queue (already processed)
    }

    /// Remove a manual_tx_id from the queue (called when admin finalizes).
    pub async fn dequeue(
        &self,
        manual_tx_id: Uuid,
        tx_type: &str,
        asset_class: &str,
    ) -> AppResult<()> {
        let mut conn = self.conn.clone();
        let list_key = self.key_pending_list(tx_type, asset_class);
        // Need to find the payload to remove. We rebuild it by scanning.
        let raw: Vec<String> = conn.lrange(&list_key, 0, -1).await?;
        for r in raw {
            if let Ok((id, _)) = serde_json::from_str::<(Uuid, Uuid)>(r) {
                if id == manual_tx_id {
                    let _: () = conn.lrem(&list_key, 1, r).await?;
                    break;
                }
            }
        }
        Ok(())
    }

    // --- pub/sub status broadcast -------------------------------------------

    /// Publish a status change to the user's channel.
    pub async fn publish_status(
        &self,
        user_id: Uuid,
        event: &QueueEvent,
    ) -> AppResult<()> {
        let mut conn = self.conn.clone();
        let channel = self.key_status_channel(user_id);
        let payload = serde_json::to_string(event)?;
        let _: () = conn.publish(&channel, payload).await?;
        Ok(())
    }

    /// Returns the channel name a user subscribes to for status updates.
    pub fn user_channel(&self, user_id: Uuid) -> String {
        self.key_status_channel(user_id)
    }

    // --- simple cache helpers used by API -----------------------------------

    pub async fn cache_set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> AppResult<()> {
        let mut conn = self.conn.clone();
        let payload = serde_json::to_string(value)?;
        let _: () = conn.set_ex(key, payload, ttl.as_secs()).await?;
        Ok(())
    }

    pub async fn cache_get<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> AppResult<Option<T>> {
        let mut conn = self.conn.clone();
        let raw: Option<String> = conn.get(key).await?;
        match raw {
            None => Ok(None),
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
        }
    }
}

/// Type alias used across modules.
pub type SharedQueue = Arc<RedisQueue>;
