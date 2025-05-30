use crate::client::Client;
use crate::named_packet::NamedPacket;
use async_trait::async_trait;
use minecraft_protocol::prelude::DecodePacket;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

#[async_trait]
pub trait Handler<S>: Send + Sync {
    async fn handle(&self, state: S, client: Client, raw_packet: NamedPacket);
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
impl<T, F, Fut, S> Handler<S> for ListenerHandler<T, F>
where
    T: DecodePacket + Send + Sync + 'static,
    F: Fn(S, Client, T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
    S: Sync + Send + 'static,
{
    async fn handle(&self, state: S, client: Client, raw_packet: NamedPacket) {
        let packet = async { raw_packet.decode(client.protocol_version().await).unwrap() }.await;
        (self.listener_fn)(state, client, packet).await;
    }
}
