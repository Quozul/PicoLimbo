use crate::server::client::Client;
use crate::server::client_inner::ClientSendPacketError;
use crate::server::named_packet::NamedPacket;
use crate::server_state::ServerState;
use async_trait::async_trait;
use minecraft_protocol::prelude::DecodePacket;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("Failed to decode packet '{0}'")]
    Protocol(String),
    #[error(transparent)]
    Client(#[from] ClientSendPacketError),
    #[error("Handler error: {0}")]
    Custom(String),
}

impl HandlerError {
    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }
}

#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle(
        &self,
        state: ServerState,
        client: Client,
        raw_packet: NamedPacket,
    ) -> Result<(), HandlerError>;
}

pub struct ListenerHandler<T, F> {
    listener_fn: Arc<F>,
    _marker: PhantomData<T>,
}

impl<T, F> ListenerHandler<T, F> {
    pub fn new(listener_fn: F) -> Self {
        Self {
            listener_fn: Arc::new(listener_fn),
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<T, F, Fut> Handler for ListenerHandler<T, F>
where
    T: DecodePacket + Send + Sync + 'static,
    F: Fn(ServerState, Client, T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), HandlerError>> + Send + 'static,
{
    async fn handle(
        &self,
        state: ServerState,
        client: Client,
        raw_packet: NamedPacket,
    ) -> Result<(), HandlerError> {
        let packet = async {
            let protocol_ver_obj = client.protocol_version().await;
            raw_packet.decode::<T>(protocol_ver_obj)
        }
        .await
        .map_err(|_| HandlerError::Protocol(raw_packet.name))?;

        (self.listener_fn)(state, client, packet).await
    }
}
