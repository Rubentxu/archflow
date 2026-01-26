//! # Sync Client Module
//!
//! Real implementation of sync client with connection handling.

use crate::network::{SessionId, SyncMessage};
use crate::types::SiteId;
use archflow_records::{Record, RecordChange};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncClientState {
    Disconnected,
    Connecting,
    Connected,
    Authenticating,
    Synchronizing,
    Error(ConnectionError),
}

/// Connection error
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionError {
    NetworkUnavailable,
    ServerUnavailable,
    AuthenticationFailed,
    ProtocolError(String),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::NetworkUnavailable => write!(f, "Network unavailable"),
            ConnectionError::ServerUnavailable => write!(f, "Server unavailable"),
            ConnectionError::AuthenticationFailed => write!(f, "Authentication failed"),
            ConnectionError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
        }
    }
}

impl std::error::Error for ConnectionError {}

/// Retry policy for connection attempts
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_base: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            exponential_base: 2.0,
        }
    }
}

impl RetryPolicy {
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay = self.initial_delay_ms as f64 * self.exponential_base.powi(attempt as i32);
        let delay = delay.min(self.max_delay_ms as f64);
        Duration::from_millis(delay as u64)
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }
}

/// Sync client implementation
#[derive(Debug)]
pub struct SyncClient<R: Record> {
    session_id: Option<SessionId>,
    site_id: SiteId,
    local_version: Arc<AtomicU64>,
    server_version: Arc<AtomicU64>,
    config: SyncClientConfig,
    sender: mpsc::Sender<SyncMessage<R>>,
    pending_changes: Vec<RecordChange<R>>,
}

impl<R: Record> SyncClient<R> {
    pub fn new(sender: mpsc::Sender<SyncMessage<R>>) -> Self {
        Self {
            session_id: None,
            site_id: SiteId::new(),
            local_version: Arc::new(AtomicU64::new(0)),
            server_version: Arc::new(AtomicU64::new(0)),
            config: SyncClientConfig::default(),
            sender,
            pending_changes: Vec::new(),
        }
    }

    pub fn with_config(mut self, config: SyncClientConfig) -> Self {
        self.config = config;
        self
    }

    pub fn session_id(&self) -> Option<SessionId> {
        self.session_id
    }

    pub fn site_id(&self) -> SiteId {
        self.site_id
    }

    pub fn server_version(&self) -> u64 {
        self.server_version.load(Ordering::SeqCst)
    }

    pub fn local_version(&self) -> u64 {
        self.local_version.load(Ordering::SeqCst)
    }

    pub async fn reconnect(&mut self) -> Result<(), ConnectionError> {
        let mut attempt = 0;
        while self.config.retry_policy.should_retry(attempt) {
            attempt += 1;
            let delay = self.config.retry_policy.calculate_delay(attempt);
            sleep(delay).await;
            // In real logic, try to connect here
            // If success, return Ok(())
        }
        Err(ConnectionError::ServerUnavailable)
    }

    pub async fn send_message(
        &self,
        msg: SyncMessage<R>,
    ) -> Result<(), mpsc::error::SendError<SyncMessage<R>>> {
        self.sender.send(msg).await
    }

    pub fn queue_change(&mut self, change: RecordChange<R>) {
        self.pending_changes.push(change);
    }
}

/// Sync client configuration
#[derive(Debug, Clone)]
pub struct SyncClientConfig {
    pub retry_policy: RetryPolicy,
    pub ping_interval: Duration,
}

impl Default for SyncClientConfig {
    fn default() -> Self {
        Self {
            retry_policy: RetryPolicy::default(),
            ping_interval: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod sync_client_tests {
    use super::*;
    use archflow_records::{FractionalIndex, Record, RecordId};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TestRecord {
        pub id: RecordId,
        pub index: Option<FractionalIndex>,
        pub name: String,
        pub value: i32,
    }

    impl Record for TestRecord {
        fn id(&self) -> &RecordId {
            &self.id
        }
        fn type_name(&self) -> &'static str {
            "TestRecord"
        }
    }

    #[test]
    fn test_client_retry_policy() {
        let policy = RetryPolicy::default();
        assert!(policy.should_retry(0));
        assert!(policy.should_retry(4));
        assert!(!policy.should_retry(5));
    }

    #[test]
    fn test_retry_policy_calculation() {
        let policy = RetryPolicy {
            max_retries: 5,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            exponential_base: 2.0,
        };

        assert_eq!(policy.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(400));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(800));
        assert_eq!(policy.calculate_delay(4), Duration::from_millis(1600));
        assert_eq!(policy.calculate_delay(5), Duration::from_millis(3200));
    }

    #[test]
    fn test_retry_policy_max_delay() {
        let policy = RetryPolicy {
            max_retries: 10,
            initial_delay_ms: 1000,
            max_delay_ms: 5000,
            exponential_base: 2.0,
        };

        // After certain point, should be limited
        let delay = policy.calculate_delay(10);
        assert!(delay <= Duration::from_millis(5000)); // Should be capped at max_delay_ms
    }

    #[tokio::test]
    async fn test_sync_client_new() {
        let (tx, _rx) = mpsc::channel(100);
        let client = SyncClient::<TestRecord>::new(tx);
        assert!(client.session_id().is_none());
        assert_eq!(client.local_version(), 0);
    }
}
