use std::collections::VecDeque;

pub struct Batch<T> {
    producers: VecDeque<Producer<T>>,
}

enum Producer<T> {
    Closure(Box<dyn FnOnce() -> T + Send + 'static>),
    Iterator(Box<dyn Iterator<Item = T> + Send + 'static>),
}

impl<T> Batch<T> {
    pub const fn new() -> Self {
        Self {
            producers: VecDeque::new(),
        }
    }

    pub fn queue<F>(&mut self, f: F)
    where
        F: FnOnce() -> T + Send + 'static,
    {
        self.producers.push_back(Producer::Closure(Box::new(f)));
    }

    pub fn chain_iter<I>(&mut self, iter: I)
    where
        I: Iterator<Item = T> + Send + 'static,
    {
        self.producers.push_back(Producer::Iterator(Box::new(iter)));
    }
}

impl<T> IntoIterator for Batch<T> {
    type Item = T;
    type IntoIter = BatchIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        BatchIterator {
            producers: self.producers,
            current_iterator: None,
        }
    }
}

pub struct BatchIterator<T> {
    producers: VecDeque<Producer<T>>,
    current_iterator: Option<Box<dyn Iterator<Item = T> + Send + 'static>>,
}

impl<T> Iterator for BatchIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut current_iter) = self.current_iterator {
                if let Some(item) = current_iter.next() {
                    return Some(item);
                }
                self.current_iterator = None;
            }

            match self.producers.pop_front() {
                Some(Producer::Closure(f)) => {
                    return Some(f());
                }
                Some(Producer::Iterator(iter)) => {
                    self.current_iterator = Some(iter);
                }
                None => {
                    return None;
                }
            }
        }
    }
}

// Ensure Batch can be sent across threads
unsafe impl<T: Send> Send for Batch<T> {}
