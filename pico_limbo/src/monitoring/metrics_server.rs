use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{error, info};

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
