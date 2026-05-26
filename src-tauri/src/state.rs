use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
struct Inner {
    active_scans: u32,
    total_completed: u64,
}

/// Shared application state injected via Tauri's state manager.
/// All access is async-safe through a tokio RwLock.
#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<RwLock<Inner>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner::default())),
        }
    }

    pub async fn get_status(&self) -> serde_json::Value {
        let s = self.inner.read().await;
        serde_json::json!({
            "active_scans":    s.active_scans,
            "total_completed": s.total_completed,
        })
    }

    pub async fn begin_scan(&self) {
        self.inner.write().await.active_scans += 1;
    }

    pub async fn end_scan(&self) {
        let mut s = self.inner.write().await;
        s.active_scans = s.active_scans.saturating_sub(1);
        s.total_completed += 1;
    }
}
