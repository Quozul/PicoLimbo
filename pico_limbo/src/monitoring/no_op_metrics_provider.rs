use crate::monitoring::metrics_provider::MetricsProvider;
use minecraft_protocol::prelude::State;

pub struct NoOpMetrics;

impl MetricsProvider for NoOpMetrics {
    fn inc_total_connections(&self) {}
    fn inc_connected_clients(&self) {}
    fn dec_connected_clients(&self) {}
    fn inc_packets_received(&self, _packet_name: &str, _state: State) {}
    fn inc_packets_sent(&self, _packet_name: &str, _state: State) {}
    fn inc_client_version(&self, _version: &str) {}
    fn inc_packet_processing_error(&self, _error_type: &str) {}
}
