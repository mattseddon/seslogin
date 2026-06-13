use std::sync::{Arc, Mutex};

use async_graphql::futures_util::future::BoxFuture;
use tokio::task::JoinHandle;

#[derive(Debug, Default)]
pub struct RequestMetrics {
    read_units: Mutex<f64>,
    write_units: Mutex<f64>,
}

impl RequestMetrics {
    pub fn record(&self, description: &str, read_units: f64, write_units: f64) {
        if read_units > 0.0 || write_units > 0.0 {
            tracing::debug!(
                "capacity {}: rcu={:.1} wcu={:.1}",
                description,
                read_units,
                write_units
            );
            *self.read_units.lock().unwrap() += read_units;
            *self.write_units.lock().unwrap() += write_units;
        }
    }

    pub fn read_units(&self) -> f64 {
        *self.read_units.lock().unwrap()
    }

    pub fn write_units(&self) -> f64 {
        *self.write_units.lock().unwrap()
    }
}

tokio::task_local! {
    pub static METRICS: Arc<RequestMetrics>;
}

/// Custom spawner for DataLoader. Propagates the METRICS task-local into each spawned
/// batch-load task so DataLoader reads are captured in the per-request accumulator.
/// Falls back to plain tokio::spawn when no metrics are active (e.g. Lambda sync contexts).
pub fn metrics_spawner(future: BoxFuture<'static, ()>) -> JoinHandle<()> {
    match METRICS.try_with(|m| m.clone()) {
        Ok(metrics) => tokio::spawn(METRICS.scope(metrics, future)),
        Err(_) => tokio::spawn(future),
    }
}
