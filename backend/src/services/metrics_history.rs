//! Metrics History Service
//!
//! Maintains historical system metrics data for dashboard graphs.
//! Uses a ring buffer to store the last N samples.

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use tokio::time::{interval, Duration};

use super::system_monitor::SystemMonitor;

/// Maximum number of samples to keep (10 minutes at 10-second intervals)
const MAX_SAMPLES: usize = 60;

/// Sampling interval in seconds
pub const SAMPLE_INTERVAL_SECS: u64 = 10;

/// A single metrics sample at a point in time
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSample {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

/// Historical metrics data
#[derive(Debug, Clone, Serialize)]
pub struct MetricsHistory {
    pub samples: Vec<MetricsSample>,
    pub sample_interval_secs: u64,
}

/// Thread-safe storage for metrics history
#[derive(Clone)]
pub struct MetricsHistoryStore {
    samples: Arc<RwLock<VecDeque<MetricsSample>>>,
}

impl Default for MetricsHistoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsHistoryStore {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_SAMPLES))),
        }
    }

    /// Add a new sample to the history
    pub fn add_sample(&self, sample: MetricsSample) {
        let mut samples = self.samples.write().unwrap();
        if samples.len() >= MAX_SAMPLES {
            samples.pop_front();
        }
        samples.push_back(sample);
    }

    /// Get all samples as a vector
    pub fn get_history(&self) -> MetricsHistory {
        let samples = self.samples.read().unwrap();
        MetricsHistory {
            samples: samples.iter().cloned().collect(),
            sample_interval_secs: SAMPLE_INTERVAL_SECS,
        }
    }

    /// Get the latest sample
    pub fn get_latest(&self) -> Option<MetricsSample> {
        let samples = self.samples.read().unwrap();
        samples.back().cloned()
    }

    /// Collect current metrics and add to history
    pub fn collect_sample(&self) {
        let metrics = SystemMonitor::get_quick_metrics();

        // Calculate primary disk usage (first disk or 0)
        let disk_usage = metrics.disks.first().map(|d| d.usage_percent).unwrap_or(0.0);

        let sample = MetricsSample {
            timestamp: Utc::now(),
            cpu_usage: metrics.cpu.usage_percent,
            memory_usage: metrics.memory.usage_percent,
            disk_usage,
            network_rx_bytes: metrics.network.received_bytes,
            network_tx_bytes: metrics.network.transmitted_bytes,
        };

        self.add_sample(sample);
    }
}

/// Global metrics history store
static METRICS_STORE: std::sync::OnceLock<MetricsHistoryStore> = std::sync::OnceLock::new();

/// Get the global metrics history store
pub fn get_metrics_store() -> &'static MetricsHistoryStore {
    METRICS_STORE.get_or_init(MetricsHistoryStore::new)
}

/// Start the background metrics collection task
pub fn start_metrics_collector() {
    let store = get_metrics_store().clone();

    // Collect initial sample immediately
    store.collect_sample();

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(SAMPLE_INTERVAL_SECS));

        loop {
            interval.tick().await;

            // Collect in blocking task to avoid blocking async runtime
            let store_clone = store.clone();
            tokio::task::spawn_blocking(move || {
                store_clone.collect_sample();
            })
            .await
            .ok();
        }
    });

    tracing::info!("Metrics collector started (interval: {}s)", SAMPLE_INTERVAL_SECS);
}
