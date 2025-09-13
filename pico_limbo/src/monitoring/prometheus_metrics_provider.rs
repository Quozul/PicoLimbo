use crate::monitoring::metrics::{ErrorLabels, Metrics, PacketLabels, VersionLabels};
use crate::monitoring::metrics_provider::MetricsProvider;
use minecraft_protocol::prelude::State;
use std::sync::Arc;

#[cfg(feature = "monitoring")]
pub struct PrometheusMetrics {
    metrics: Arc<Metrics>,
}

#[cfg(feature = "monitoring")]
impl PrometheusMetrics {
    pub const fn new(metrics: Arc<Metrics>) -> Self {
        Self { metrics }
    }
}

#[cfg(feature = "monitoring")]
impl MetricsProvider for PrometheusMetrics {
    fn inc_total_connections(&self) {
        self.metrics.total_connections.inc();
    }

    fn inc_connected_clients(&self) {
        self.metrics.connected_clients.inc();
    }

    fn dec_connected_clients(&self) {
        self.metrics.connected_clients.dec();
    }

    fn inc_packets_received(&self, packet_name: &str, state: State) {
        self.metrics
            .packet_traffic_total
            .get_or_create(&PacketLabels {
                name: packet_name.to_string(),
                state: state.to_string(),
                direction: "received".to_string(),
            })
            .inc();
    }

    fn inc_packets_sent(&self, packet_name: &str, state: State) {
        self.metrics
            .packet_traffic_total
            .get_or_create(&PacketLabels {
                name: packet_name.to_string(),
                state: state.to_string(),
                direction: "sent".to_string(),
            })
            .inc();
    }

    fn inc_client_version(&self, version: &str) {
        self.metrics
            .client_versions
            .get_or_create(&VersionLabels {
                version: version.to_string(),
            })
            .inc();
    }

    fn inc_packet_processing_error(&self, error_type: &str) {
        self.metrics
            .packet_processing_errors
            .get_or_create(&ErrorLabels {
                error_type: error_type.to_string(),
            })
            .inc();
    }
}
