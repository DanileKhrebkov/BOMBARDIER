// src/metrics/collector.rs
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Metric {
    pub step_name: String,
    pub latency: Duration,
    pub status: u16,
    pub success: bool,
    pub timestamp: Instant,
}

pub type MetricSender = mpsc::UnboundedSender<Metric>;
pub type MetricReceiver = mpsc::UnboundedReceiver<Metric>;

#[derive(Clone)]
pub struct MetricsCollector {
    sender: MetricSender,
    receiver: Arc<DashMap<usize, MetricReceiver>>,
}

impl MetricsCollector {
    pub fn new() -> (Self, MetricReceiver) {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let collector = Self {
            sender: tx,
            receiver: Arc::new(DashMap::new()),
        };
        
        (collector, rx)
    }

    pub fn get_sender(&self) -> MetricSender {
        self.sender.clone()
    }
}