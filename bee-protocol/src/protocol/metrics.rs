use std::sync::atomic::{
    AtomicU32,
    Ordering,
};

#[derive(Default)]
pub struct ProtocolMetrics {
    invalid_transactions_received: AtomicU32,
    stale_transactions_received: AtomicU32,
    random_transactions_received: AtomicU32,
    new_transactions_received: AtomicU32,
    known_transactions_received: AtomicU32,

    invalid_messages_received: AtomicU32,

    milestone_request_received: AtomicU32,
    transaction_broadcast_received: AtomicU32,
    transaction_request_received: AtomicU32,
    heartbeat_received: AtomicU32,

    milestone_request_sent: AtomicU32,
    transaction_broadcast_sent: AtomicU32,
    transaction_request_sent: AtomicU32,
    heartbeat_sent: AtomicU32,
}

impl ProtocolMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ProtocolMetrics {
    pub fn invalid_transactions_received(&self) -> u32 {
        self.invalid_transactions_received.load(Ordering::Relaxed)
    }

    pub fn invalid_transactions_received_inc(&self) -> u32 {
        self.invalid_transactions_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn stale_transactions_received(&self) -> u32 {
        self.stale_transactions_received.load(Ordering::Relaxed)
    }

    pub fn stale_transactions_received_inc(&self) -> u32 {
        self.stale_transactions_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn random_transactions_received(&self) -> u32 {
        self.random_transactions_received.load(Ordering::Relaxed)
    }

    pub fn random_transactions_received_inc(&self) -> u32 {
        self.random_transactions_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn new_transactions_received(&self) -> u32 {
        self.new_transactions_received.load(Ordering::Relaxed)
    }

    pub fn new_transactions_received_inc(&self) -> u32 {
        self.new_transactions_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn known_transactions_received(&self) -> u32 {
        self.known_transactions_received.load(Ordering::Relaxed)
    }

    pub fn known_transactions_received_inc(&self) -> u32 {
        self.known_transactions_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn invalid_messages_received(&self) -> u32 {
        self.invalid_messages_received.load(Ordering::Relaxed)
    }

    pub fn invalid_messages_received_inc(&self) -> u32 {
        self.invalid_messages_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn milestone_request_received(&self) -> u32 {
        self.milestone_request_received.load(Ordering::Relaxed)
    }

    pub fn milestone_request_received_inc(&self) -> u32 {
        self.milestone_request_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn transaction_broadcast_received(&self) -> u32 {
        self.transaction_broadcast_received.load(Ordering::Relaxed)
    }

    pub fn transaction_broadcast_received_inc(&self) -> u32 {
        self.transaction_broadcast_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn transaction_request_received(&self) -> u32 {
        self.transaction_request_received.load(Ordering::Relaxed)
    }

    pub fn transaction_request_received_inc(&self) -> u32 {
        self.transaction_request_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn heartbeat_received(&self) -> u32 {
        self.heartbeat_received.load(Ordering::Relaxed)
    }

    pub fn heartbeat_received_inc(&self) -> u32 {
        self.heartbeat_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn milestone_request_sent(&self) -> u32 {
        self.milestone_request_sent.load(Ordering::Relaxed)
    }

    pub fn milestone_request_sent_inc(&self) -> u32 {
        self.milestone_request_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn transaction_broadcast_sent(&self) -> u32 {
        self.transaction_broadcast_sent.load(Ordering::Relaxed)
    }

    pub fn transaction_broadcast_sent_inc(&self) -> u32 {
        self.transaction_broadcast_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn transaction_request_sent(&self) -> u32 {
        self.transaction_request_sent.load(Ordering::Relaxed)
    }

    pub fn transaction_request_sent_inc(&self) -> u32 {
        self.transaction_request_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn heartbeat_sent(&self) -> u32 {
        self.heartbeat_sent.load(Ordering::Relaxed)
    }

    pub fn heartbeat_sent_inc(&self) -> u32 {
        self.heartbeat_sent.fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn node_metrics_transactions_received_test() {
        let metrics = ProtocolMetrics::default();

        assert_eq!(metrics.invalid_transactions_received(), 0);
        assert_eq!(metrics.stale_transactions_received(), 0);
        assert_eq!(metrics.random_transactions_received(), 0);
        assert_eq!(metrics.new_transactions_received(), 0);
        assert_eq!(metrics.known_transactions_received(), 0);

        metrics.invalid_transactions_received_inc();
        metrics.stale_transactions_received_inc();
        metrics.random_transactions_received_inc();
        metrics.new_transactions_received_inc();
        metrics.known_transactions_received_inc();

        assert_eq!(metrics.invalid_transactions_received(), 1);
        assert_eq!(metrics.stale_transactions_received(), 1);
        assert_eq!(metrics.random_transactions_received(), 1);
        assert_eq!(metrics.new_transactions_received(), 1);
        assert_eq!(metrics.known_transactions_received(), 1);
    }

    #[test]
    fn node_metrics_messages_received_test() {
        let metrics = ProtocolMetrics::default();

        assert_eq!(metrics.invalid_messages_received(), 0);
        assert_eq!(metrics.milestone_request_received(), 0);
        assert_eq!(metrics.transaction_broadcast_received(), 0);
        assert_eq!(metrics.transaction_request_received(), 0);
        assert_eq!(metrics.heartbeat_received(), 0);

        metrics.invalid_messages_received_inc();
        metrics.milestone_request_received_inc();
        metrics.transaction_broadcast_received_inc();
        metrics.transaction_request_received_inc();
        metrics.heartbeat_received_inc();

        assert_eq!(metrics.invalid_messages_received(), 1);
        assert_eq!(metrics.milestone_request_received(), 1);
        assert_eq!(metrics.transaction_broadcast_received(), 1);
        assert_eq!(metrics.transaction_request_received(), 1);
        assert_eq!(metrics.heartbeat_received(), 1);
    }

    #[test]
    fn node_metrics_messages_sent_test() {
        let metrics = ProtocolMetrics::default();

        assert_eq!(metrics.milestone_request_sent(), 0);
        assert_eq!(metrics.transaction_broadcast_sent(), 0);
        assert_eq!(metrics.transaction_request_sent(), 0);
        assert_eq!(metrics.heartbeat_sent(), 0);

        metrics.milestone_request_sent_inc();
        metrics.transaction_broadcast_sent_inc();
        metrics.transaction_request_sent_inc();
        metrics.heartbeat_sent_inc();

        assert_eq!(metrics.milestone_request_sent(), 1);
        assert_eq!(metrics.transaction_broadcast_sent(), 1);
        assert_eq!(metrics.transaction_request_sent(), 1);
        assert_eq!(metrics.heartbeat_sent(), 1);
    }
}