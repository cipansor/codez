//! Port of `modules/util/timer.go`.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// `util.Debounce` — returns a sink that runs `f` only after `d` has passed
/// with no further call.
///
/// The Go version closes over a `*time.Timer` guarded by a mutex and uses the
/// timer identity to cancel a stale fire. The Rust equivalent uses a generation
/// counter under the same lock: a pending fire only runs when it is still the
/// newest one, which is exactly what `trigger == db.t` checks upstream.
pub struct Debouncer {
    delay: Duration,
    state: Arc<Mutex<DebounceState>>,
}

#[derive(Default)]
struct DebounceState {
    generation: u64,
}

impl Debouncer {
    /// Builds a debouncer that waits `delay` between the last call and the fire.
    pub fn new(delay: Duration) -> Self {
        Self {
            delay,
            state: Arc::new(Mutex::new(DebounceState::default())),
        }
    }

    /// `util.Debounce`'s returned function — schedules `f` after the delay,
    /// cancelling any earlier pending call.
    pub fn call<F: FnOnce() + Send + 'static>(&self, f: F) {
        let generation = {
            let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
            st.generation += 1;
            st.generation
        };

        let state = Arc::clone(&self.state);
        let delay = self.delay;
        thread::spawn(move || {
            thread::sleep(delay);
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            // Only the newest scheduled call fires; earlier ones are stale.
            if st.generation == generation {
                drop(st);
                f();
            }
        });
    }
}

/// `time.Since` helper used by callers that need elapsed time.
pub fn elapsed_since(start: Instant) -> Duration {
    start.elapsed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn only_the_last_call_fires() {
        let (tx, rx) = mpsc::channel::<u32>();
        let d = Debouncer::new(Duration::from_millis(40));

        for i in 0..5 {
            let tx = tx.clone();
            d.call(move || {
                let _ = tx.send(i);
            });
            thread::sleep(Duration::from_millis(5));
        }

        let got = rx.recv_timeout(Duration::from_secs(2)).expect("a fire");
        assert_eq!(got, 4, "only the newest scheduled call should run");
        assert!(
            rx.recv_timeout(Duration::from_millis(120)).is_err(),
            "earlier calls must be cancelled"
        );
    }

    #[test]
    fn a_single_call_still_fires() {
        let (tx, rx) = mpsc::channel::<()>();
        let d = Debouncer::new(Duration::from_millis(20));
        d.call(move || {
            let _ = tx.send(());
        });
        assert!(rx.recv_timeout(Duration::from_secs(2)).is_ok());
    }
}
