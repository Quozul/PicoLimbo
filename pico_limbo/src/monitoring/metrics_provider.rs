pub trait MetricsProvider: Send + Sync {
    fn inc_total_connections(&self);
    fn inc_connected_clients(&self);
    fn dec_connected_clients(&self);
    fn inc_packets_received(&self);
    fn inc_packets_sent(&self);
    fn inc_client_version(&self, version: &str);
    fn inc_packet_processing_error(&self, error_type: &str);
}
