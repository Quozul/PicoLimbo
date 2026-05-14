use crate::server::packet_registry::PacketRegistry;
use futures::stream::Stream;
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
        let closure = move || -> Pin<Box<dyn Future<Output = PacketRegistry> + Send>> { Box::pin(f()) };
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

impl Stream for BatchStream {
    type Item = PacketRegistry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            match &mut this.current {
                Current::Future(fut) => {
                    return match fut.as_mut().poll(cx) {
                        Poll::Ready(item) => {
                            this.current = Current::Idle;
                            Poll::Ready(Some(item))
                        }
                        Poll::Pending => Poll::Pending,
                    };
                }
                Current::Iterator(iter) => {
                    if let Some(item) = iter.next() {
                        return Poll::Ready(Some(item));
                    }
                    this.current = Current::Idle;
                }
                Current::Idle => match this.producers.pop_front() {
                    Some(Producer::SyncClosure(f)) => {
                        return Poll::Ready(Some(f()));
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
