use crate::monitoring::metrics_provider::MetricsProvider;
use std::sync::Arc;

#[cfg(feature = "monitoring")]
mod metrics;
pub mod metrics_provider;
#[cfg(feature = "monitoring")]
mod metrics_server;
#[cfg(not(feature = "monitoring"))]
mod no_op_metrics_provider;
#[cfg(feature = "monitoring")]
mod prometheus_metrics_provider;

pub fn get_metrics_provider() -> Arc<dyn MetricsProvider> {
    #[cfg(feature = "monitoring")]
    {
        use crate::monitoring::metrics::Metrics;
        use crate::monitoring::metrics_server::MetricsServer;
        use crate::monitoring::prometheus_metrics_provider::PrometheusMetrics;
        use prometheus_client::registry::Registry;
        use tokio::sync::Mutex;

        let mut registry = Registry::default();
        let metrics = Arc::new(Metrics::new(&mut registry));
        let registry = Arc::new(Mutex::new(registry));
        let metrics_server = MetricsServer::new("0.0.0.0:9090", Arc::clone(&registry));

        tokio::spawn(async move {
            metrics_server.run().await;
        });

        Arc::new(PrometheusMetrics::new(metrics))
    }
    #[cfg(not(feature = "monitoring"))]
    {
        use crate::monitoring::no_op_metrics_provider::NoOpMetrics;

        Arc::new(NoOpMetrics)
    }
}
