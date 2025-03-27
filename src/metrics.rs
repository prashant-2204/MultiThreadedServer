use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

pub struct Metrics {
    requests: AtomicU64,
    latency_histogram: RwLock<Vec<f64>>,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            requests: AtomicU64::new(0),
            latency_histogram: RwLock::new(Vec::new()),
        }
    }

    pub fn increment_requests(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_latency(&self, latency: f64) {
        let mut hist = self.latency_histogram.write().unwrap();
        if hist.len() >= 1000 {
            hist.remove(0);
        }
        hist.push(latency);
    }

    pub fn get_requests(&self) -> u64 {
        self.requests.load(Ordering::Relaxed)
    }
}