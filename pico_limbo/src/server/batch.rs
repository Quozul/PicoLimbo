use crate::server::packet_registry::PacketRegistry;
use futures::stream::Stream;
use minecraft_protocol::prelude::{Direction, State};
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

type AsyncClosure =
    Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = PacketRegistry> + Send>> + Send + 'static>;

enum Producer {
    SyncClosure(Box<dyn FnOnce() -> PacketRegistry + Send + 'static>),
    AsyncClosure(AsyncClosure),
    Iterator(Box<dyn Iterator<Item = PacketRegistry> + Send + 'static>),
    StateChange(Direction, State),
    EnableCompression,
}

pub struct Batch {
    producers: VecDeque<Producer>,
}

impl Batch {
    pub const fn new() -> Self {
        Self {
            producers: VecDeque::new(),
        }
    }

    pub fn queue_enable_compression(&mut self) {
        self.producers.push_back(Producer::EnableCompression);
    }

    /// Queue a state change for both directions. All received and sent packets will use the new state.
    pub fn queue_both_state_change(&mut self, new_state: State) {
        self.queue_clientbound_state_change(new_state);
        self.queue_serverbound_state_change(new_state);
    }

    /// Queue a state change for sent packets.
    pub fn queue_clientbound_state_change(&mut self, new_state: State) {
        self.producers
            .push_back(Producer::StateChange(Direction::Clientbound, new_state));
    }

    /// Queue a state change for received packets.
    pub fn queue_serverbound_state_change(&mut self, new_state: State) {
        self.producers
            .push_back(Producer::StateChange(Direction::Serverbound, new_state));
    }

    /// Queues a synchronous function or closure.
    pub fn queue<F>(&mut self, f: F)
    where
        F: FnOnce() -> PacketRegistry + Send + 'static,
    {
        self.producers.push_back(Producer::SyncClosure(Box::new(f)));
    }

    /// Queues an async closure that may or may not produce a value.
    pub fn queue_async<F, Fut>(&mut self, f: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = PacketRegistry> + Send + 'static,
    {
        let closure =
            move || -> Pin<Box<dyn Future<Output = PacketRegistry> + Send>> { Box::pin(f()) };
        self.producers
            .push_back(Producer::AsyncClosure(Box::new(closure)));
    }

    /// Chains a synchronous iterator.
    pub fn chain_iter<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = PacketRegistry>,
        I::IntoIter: Send + 'static,
    {
        self.producers
            .push_back(Producer::Iterator(Box::new(iter.into_iter())));
    }

    pub fn into_stream(self) -> BatchStream {
        BatchStream {
            producers: self.producers,
            current: Current::Idle,
        }
    }
}

enum Current {
    Idle,
    Future(Pin<Box<dyn Future<Output = PacketRegistry> + Send>>),
    Iterator(Box<dyn Iterator<Item = PacketRegistry> + Send>),
}

pub struct BatchStream {
    producers: VecDeque<Producer>,
    current: Current,
}

pub enum BatchItem {
    Packet(PacketRegistry),
    StateChange(Direction, State),
    EnableCompression,
}

impl Stream for BatchStream {
    type Item = BatchItem;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            match &mut this.current {
                Current::Future(fut) => {
                    return match fut.as_mut().poll(cx) {
                        Poll::Ready(item) => {
                            this.current = Current::Idle;
                            Poll::Ready(Some(BatchItem::Packet(item)))
                        }
                        Poll::Pending => Poll::Pending,
                    };
                }
                Current::Iterator(iter) => {
                    if let Some(item) = iter.next() {
                        return Poll::Ready(Some(BatchItem::Packet(item)));
                    }
                    this.current = Current::Idle;
                }
                Current::Idle => match this.producers.pop_front() {
                    Some(Producer::SyncClosure(f)) => {
                        return Poll::Ready(Some(BatchItem::Packet(f())));
                    }
                    Some(Producer::StateChange(direction, new_state)) => {
                        return Poll::Ready(Some(BatchItem::StateChange(direction, new_state)));
                    }
                    Some(Producer::EnableCompression) => {
                        return Poll::Ready(Some(BatchItem::EnableCompression));
                    }
                    Some(Producer::AsyncClosure(f)) => {
                        this.current = Current::Future(f());
                    }
                    Some(Producer::Iterator(iter)) => {
                        this.current = Current::Iterator(iter);
                    }
                    None => {
                        return Poll::Ready(None);
                    }
                },
            }
        }
    }
}

#[cfg(test)]
impl BatchItem {
    pub fn unwrap_packet(&self) -> &PacketRegistry {
        match self {
            Self::Packet(packet) => packet,
            Self::StateChange(direction, state) => panic!(
                "tried to unwrap a packet, but got a state change instead direction={direction}, state={state}"
            ),
            Self::EnableCompression => {
                panic!("tried to unwrap a packet, but got a compression instead")
            }
        }
    }

    pub fn unwrap_state_change(&self) -> (&Direction, &State) {
        match self {
            Self::StateChange(direction, state) => (direction, state),
            Self::Packet(_) => panic!("tried to unwrap a packet, but got a packet instead"),
            Self::EnableCompression => {
                panic!("tried to unwrap a packet, but got a compression instead")
            }
        }
    }
}

#[cfg(test)]
impl BatchStream {
    pub async fn assert_client_state(&mut self, state: State) {
        use futures::StreamExt;
        let v = self.next().await.unwrap();
        let (direction, s) = v.unwrap_state_change();
        assert_eq!(direction, &Direction::Clientbound);
        assert_eq!(s, &state);
    }

    pub async fn assert_server_state(&mut self, state: State) {
        use futures::StreamExt;
        let v = self.next().await.unwrap();
        let (direction, s) = v.unwrap_state_change();
        assert_eq!(direction, &Direction::Serverbound);
        assert_eq!(s, &state);
    }
}
