use std::collections::VecDeque;

#[derive(Default)]
pub struct Fifo<T> {
    queue: VecDeque<T>,
}

pub struct FifoIntoIter<T> {
    queue: VecDeque<T>,
}

impl<T> Iterator for FifoIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop_front()
    }
}

impl<T> Fifo<T> {
    pub const fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, item: T) {
        self.queue.push_back(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.queue.pop_front()
    }

    pub fn drain(&mut self) -> impl Iterator<Item = T> + '_ {
        std::iter::from_fn(move || self.pop())
    }
}

impl<T> IntoIterator for Fifo<T> {
    type Item = T;
    type IntoIter = FifoIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        FifoIntoIter { queue: self.queue }
    }
}

impl<'a, T> IntoIterator for &'a Fifo<T> {
    type Item = &'a T;
    type IntoIter = std::collections::vec_deque::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.queue.iter()
    }
}
