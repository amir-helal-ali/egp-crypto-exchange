//! ناقل أحداث WebSocket لكل مستخدم
//! Per-user WebSocket event bus
//!
//! يحافظ على قائمة من senders لكل مستخدم متصل (يدعم عدة تبويبات).
//! When an event occurs (order filled, wallet updated, etc.), the backend
//! calls `WsBus::emit_to_user(user_id, msg)` and all of that user's open
//! WebSocket connections receive it instantly — no polling required.

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use serde_json::Value;
use tokio::sync::mpsc;
use tracing::debug;

/// قناة واحدة لكل اتصال WebSocket مفتوح.
pub type WsSender = mpsc::UnboundedSender<Value>;

/// ناقل الأحداث المركزي — يربط الأحداث الخلفية باتصالات WS لكل مستخدم.
pub struct WsBus {
    /// user_id -> قائمة senders (واحد لكل تبويب مفتوح)
    senders: DashMap<uuid::Uuid, Vec<WsSender>>,
}

impl WsBus {
    pub fn new() -> Self {
        Self {
            senders: DashMap::new(),
        }
    }

    /// تسجيل اتصال جديد للمستخدم. يعيد receiver يستقبل الرسائل.
    pub fn register(&self, user_id: uuid::Uuid) -> mpsc::UnboundedReceiver<Value> {
        let (tx, rx) = mpsc::unbounded_channel::<Value>();
        self.senders
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(tx);
        debug!(%user_id, "WS bus: registered connection");
        rx
    }

    /// إلغاء تسجيل اتصال (عند إغلاق WS).
    pub fn unregister(&self, user_id: uuid::Uuid, target_rx: &mpsc::UnboundedReceiver<Value>) {
        // بطريقة بسيطة: نحذف أي senders مغلقة
        if let Some(mut senders) = self.senders.get_mut(&user_id) {
            senders.retain(|s| !s.is_closed());
            if senders.is_empty() {
                drop(senders);
                self.senders.remove(&user_id);
            }
        }
        debug!(%user_id, "WS bus: unregistered connection");
    }

    /// إرسال رسالة لكل اتصالات مستخدم معين.
    pub fn emit_to_user(&self, user_id: uuid::Uuid, msg: Value) {
        if let Some(mut senders) = self.senders.get_mut(&user_id) {
            // إزالة senders المغلقة أثناء الإرسال
            senders.retain(|s| {
                if s.is_closed() {
                    false
                } else {
                    let _ = s.send(msg.clone());
                    true
                }
            });
            if senders.is_empty() {
                drop(senders);
                self.senders.remove(&user_id);
            }
        }
    }

    /// بث رسالة لكل المستخدمين المتصلين (للأحداث العامة).
    pub fn broadcast(&self, msg: Value) {
        for entry in self.senders.iter() {
            for s in entry.value().iter() {
                let _ = s.send(msg.clone());
            }
        }
    }

    /// عدد المستخدمين المتصلين حالياً.
    pub fn connected_users(&self) -> usize {
        self.senders.len()
    }
}

pub type SharedWsBus = Arc<WsBus>;
