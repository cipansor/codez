//! Port of `modules/util/sync.go`.

use std::sync::{Arc, Mutex};

/// `util.OnceValue` — like `std::sync::OnceLock`, but resettable.
///
/// Upstream returns the value by copy and re-raises a panicking initialiser on
/// every call. This port hands out an `Arc<T>` instead of a copy: it keeps the
/// "initialise once, share the result" contract without requiring `T: Clone`,
/// and it stays sound under concurrent `value()`/`reset()` because a caller
/// holding an `Arc` keeps the value alive even if another thread resets.
pub struct OnceValue<T> {
    func: Box<dyn Fn() -> T + Send + Sync>,
    slot: Mutex<Option<Arc<T>>>,
}

impl<T> OnceValue<T> {
    /// Builds a resettable once-value around `func`.
    pub fn new(func: impl Fn() -> T + Send + Sync + 'static) -> Self {
        Self {
            func: Box::new(func),
            slot: Mutex::new(None),
        }
    }

    /// `OnceValue.Value` — computes on first call, then returns the cached value.
    ///
    /// The initialiser runs while the lock is held, so two racing callers see
    /// exactly one execution, matching upstream's double-checked behaviour.
    pub fn value(&self) -> Arc<T> {
        let mut guard = self.slot.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(v) = guard.as_ref() {
            return Arc::clone(v);
        }
        let v = Arc::new((self.func)());
        *guard = Some(Arc::clone(&v));
        v
    }

    /// `OnceValue.Reset` — drops the cached value so the next call recomputes.
    ///
    /// Callers that already hold an `Arc` from a previous [`OnceValue::value`]
    /// keep a valid value; only future calls recompute.
    pub fn reset(&self) {
        let mut guard = self.slot.lock().unwrap_or_else(|e| e.into_inner());
        *guard = None;
    }
}

/// `util.OnceValue` is shared across threads upstream; keep that here.
pub type SharedOnce<T> = Arc<OnceValue<T>>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Transcribed from `TestOnceValue/RepeatCall` in modules/util/sync_test.go.
    #[test]
    fn repeat_call_computes_once_then_reuses() {
        let call_count = AtomicUsize::new(0);
        let counter = Arc::new(call_count);
        let c = Arc::clone(&counter);
        let o = OnceValue::new(move || {
            c.fetch_add(1, Ordering::SeqCst);
            42
        });

        assert_eq!(*o.value(), 42);
        assert_eq!(*o.value(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        o.reset();
        assert_eq!(*o.value(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
        assert_eq!(*o.value(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn reset_before_first_call_is_harmless() {
        let o = OnceValue::new(|| 7);
        o.reset();
        assert_eq!(*o.value(), 7);
        assert_eq!(*o.value(), 7);
    }

    /// A value handed out before a reset stays valid afterwards.
    #[test]
    fn outstanding_handle_survives_reset() {
        let o = OnceValue::new(|| String::from("first"));
        let held = o.value();
        o.reset();
        let fresh = o.value();
        assert_eq!(&*held, "first");
        assert_eq!(&*fresh, "first");
        assert_eq!(o.value().as_str(), "first");
    }
}
