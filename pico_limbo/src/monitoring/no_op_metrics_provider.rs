use crate::monitoring::metrics_provider::MetricsProvider;

pub struct NoOpMetrics;

impl MetricsProvider for NoOpMetrics {
    fn inc_total_connections(&self) {}
    fn inc_connected_clients(&self) {}
    fn dec_connected_clients(&self) {}
    fn inc_packets_received(&self) {}
    fn inc_packets_sent(&self) {}
    fn inc_client_version(&self, _version: &str) {}
    fn inc_packet_processing_error(&self, _error_type: &str) {}
}
