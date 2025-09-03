use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{error, info};

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ErrorLabels {
    pub error_type: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct VersionLabels {
    pub version: String,
}

pub struct Metrics {
    pub connected_clients: Gauge,
    pub total_connections: Counter,
    pub packets_received: Counter,
    pub packets_sent: Counter,
    pub packet_processing_errors: Family<ErrorLabels, Counter>,
    pub client_versions: Family<VersionLabels, Gauge>,
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
        let packets_received = Counter::default();
        registry.register(
            "mc_packets_received_total",
            "Total number of packets received from clients",
            packets_received.clone(),
        );
        let packets_sent = Counter::default();
        registry.register(
            "mc_packets_sent_total",
            "Total number of packets sent to clients",
            packets_sent.clone(),
        );
        let packet_processing_errors = Family::<ErrorLabels, Counter>::default();
        registry.register(
            "mc_packet_processing_errors_total",
            "Total number of packet processing errors by type",
            packet_processing_errors.clone(),
        );

        let client_versions = Family::<VersionLabels, Gauge>::default();
        registry.register(
            "mc_client_versions_connected",
            "Number of currently connected clients by Minecraft version",
            client_versions.clone(),
        );
        Self {
            connected_clients,
            total_connections,
            packets_received,
            packets_sent,
            packet_processing_errors,
            client_versions,
        }
    }
}

pub struct MetricsServer {
    listen_address: String,
    registry: Arc<Mutex<Registry>>,
}

impl MetricsServer {
    pub fn new(listen_address: &str, registry: Arc<Mutex<Registry>>) -> Self {
        Self {
            listen_address: listen_address.to_string(),
            registry,
        }
    }

    pub async fn run(self) {
        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .with_state(self.registry);

        info!("Metrics server listening on: {}", self.listen_address);
        let Ok(listener) = TcpListener::bind(&self.listen_address).await else {
            error!("Failed to bind metrics server to {}", self.listen_address);
            return;
        };

        axum::serve(listener, app)
            .await
            .unwrap_or_else(|e| error!("Metrics server failed: {}", e));
    }
}

pub async fn metrics_handler(State(registry): State<Arc<Mutex<Registry>>>) -> impl IntoResponse {
    let mut buffer = String::new();
    let registry = registry.lock().await;
    encode(&mut buffer, &registry).unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )
        .body(Body::from(buffer))
        .unwrap()
}
