use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ErrorLabels {
    pub error_type: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct VersionLabels {
    pub version: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct PacketLabels {
    pub name: String,
    pub state: String,
    pub direction: String,
}

pub struct Metrics {
    pub connected_clients: Gauge,
    pub total_connections: Counter,
    pub packet_traffic_total: Family<PacketLabels, Counter>,
    pub packet_processing_errors: Family<ErrorLabels, Counter>,
    pub client_versions: Family<VersionLabels, Counter>,
}

impl Metrics {
    pub fn new(registry: &mut Registry) -> Self {
        let connected_clients = Gauge::default();
        registry.register(
            "mc_connected_clients_total",
            "Number of clients currently connected",
            connected_clients.clone(),
        );
        let total_connections = Counter::default();
        registry.register(
            "mc_connections_total",
            "Total number of connections handled since startup",
            total_connections.clone(),
        );
        let packet_traffic_total = Family::<PacketLabels, Counter>::default();
        registry.register(
            "mc_packet_traffic_total",
            "Total number of packets sent and received, by name, state, and direction",
            packet_traffic_total.clone(),
        );
        let packet_processing_errors = Family::<ErrorLabels, Counter>::default();
        registry.register(
            "mc_packet_processing_errors_total",
            "Total number of packet processing errors by type",
            packet_processing_errors.clone(),
        );
        let client_versions = Family::<VersionLabels, Counter>::default();
        registry.register(
            "mc_client_versions_connected",
            "Number of currently connected clients by Minecraft version",
            client_versions.clone(),
        );
        Self {
            connected_clients,
            total_connections,
            packet_traffic_total,
            packet_processing_errors,
            client_versions,
        }
    }
}
