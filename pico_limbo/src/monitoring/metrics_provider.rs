use minecraft_protocol::prelude::State;

pub trait MetricsProvider: Send + Sync {
    fn inc_total_connections(&self);
    fn inc_connected_clients(&self);
    fn dec_connected_clients(&self);
    fn inc_packets_received(&self, packet_name: &str, state: State);
    fn inc_packets_sent(&self, packet_name: &str, state: State);
    fn inc_client_version(&self, version: &str);
    fn inc_packet_processing_error(&self, error_type: &str);
}
