use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tokio::time::{Duration, Instant, Interval, MissedTickBehavior};

struct IntervalState {
    interval: Option<Interval>,
}

/// A controllable interval that can be enabled, disabled, and have its
/// duration changed on the fly.
///
/// It is designed to be cloneable and safe to share across multiple
/// asynchronous tasks.
#[derive(Clone)]
pub struct ControllableInterval {
    state: Arc<Mutex<IntervalState>>,
    notify: Arc<Notify>,
}

impl ControllableInterval {
    /// Creates a new `ControllableInterval`, initially disabled.
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(IntervalState { interval: None })),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Disables the interval and sets a new one that starts ticking immediately.
    ///
    /// Any task currently waiting on `tick()` will be woken up and will
    /// start waiting for the new interval.
    pub async fn set_interval(&self, period: Duration) {
        let mut state = self.state.lock().await;
        let mut new_interval = tokio::time::interval(period);
        new_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        state.interval = Some(new_interval);
        self.notify.notify_waiters();
    }

    /// Disables the interval and sets a new one to begin at a specific time.
    ///
    /// The first tick will occur at or after `start`, and subsequent ticks will
    /// occur every `period` after that.
    ///
    /// Any task currently waiting on `tick()` will be woken up and will
    /// start waiting for the new interval to begin.
    pub async fn set_interval_at(&self, start: Instant, period: Duration) {
        let mut state = self.state.lock().await;
        let mut new_interval = tokio::time::interval_at(start, period);
        new_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        state.interval = Some(new_interval);
        self.notify.notify_waiters();
    }

    /// Disables the interval.
    ///
    /// Any task currently waiting on `tick()` will be woken up and will
    /// wait indefinitely until a new interval is set.
    pub async fn clear_interval(&self) {
        let mut state = self.state.lock().await;
        state.interval = None;
        // Notify any waiting tasks that the interval has been cleared.
        self.notify.notify_waiters();
    }

    /// Waits for the next tick of the interval.
    ///
    /// - If the interval is enabled, this method will complete when the next
    ///   tick is due.
    /// - If the interval is disabled, this method will wait until the interval
    ///   is enabled via `set_interval` and then wait for its first tick.
    pub async fn tick(&self) {
        loop {
            let notified = self.notify.notified();

            {
                let mut state = self.state.lock().await;

                if let Some(interval) = state.interval.as_mut() {
                    tokio::select! {
                        biased;

                        _ = interval.tick() => {
                            break;
                        }
                        _ = notified => {
                            continue;
                        }
                    }
                }
            }

            notified.await;
        }
    }
}
