use crate::connection_handler::ConnectionHandler;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{error, info};

pub struct Server<C: ConnectionHandler + Send + Sync + 'static> {
    listener: TcpListener,
    connection_handler: Arc<C>,
}

impl<C: ConnectionHandler + Send + Sync + 'static> Server<C> {
    pub async fn new(listen_address: String, connection_handler: C) -> std::io::Result<Self> {
        let listener = TcpListener::bind(&listen_address).await?;
        info!("Listening on: {}", listen_address);

        Ok(Self {
            listener,
            connection_handler: Arc::new(connection_handler),
        })
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                _ = self.accept() => {},
                _ = signal::ctrl_c() => {
                    info!("SIGINT received, shutting down gracefully.");
                    break;
                },
            }
        }

        self.connection_handler.on_stop().await?;

        Ok(())
    }

    async fn accept(&self) {
        let accept_result = self.listener.accept().await;
        match accept_result {
            Ok((stream, addr)) => {
                info!("Accepted connection from: {}", addr);
                let connection_handler = self.connection_handler.clone();

                tokio::spawn(async move {
                    if let Err(err) = connection_handler.on_accept(stream, addr).await {
                        error!("Error handling client {}: {:?}", addr, err);
                    }
                });
            }
            Err(err) => {
                error!("Error accepting connection: {:?}", err);
            }
        }
    }
}
