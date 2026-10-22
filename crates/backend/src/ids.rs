//! Identifier allocation shared by every store.

use std::sync::atomic::{AtomicI64, Ordering};

/// Monotonic `i64` id source.
///
/// Gitea ids are `int64` and never reused, so a simple counter is faithful
/// enough for the in-memory implementation.
#[derive(Debug)]
pub struct IdGen(AtomicI64);

impl IdGen {
    /// Start allocating from `start`.
    pub const fn starting_at(start: i64) -> Self {
        Self(AtomicI64::new(start))
    }

    /// Allocate the next id.
    pub fn next(&self) -> i64 {
        self.0.fetch_add(1, Ordering::Relaxed)
    }

    /// The most recently allocated id, or `start - 1` when none were.
    pub fn last(&self) -> i64 {
        self.0.load(Ordering::Relaxed) - 1
    }

    /// Ensure the next allocation is at least `min`, used after seeding rows
    /// with hand-written ids.
    pub fn ensure_at_least(&self, min: i64) {
        let mut current = self.0.load(Ordering::Relaxed);
        while current < min {
            match self
                .0
                .compare_exchange(current, min, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => return,
                Err(actual) => current = actual,
            }
        }
    }
}

impl Default for IdGen {
    fn default() -> Self {
        Self::starting_at(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_monotonically() {
        let ids = IdGen::starting_at(100);
        assert_eq!(ids.next(), 100);
        assert_eq!(ids.next(), 101);
        assert_eq!(ids.last(), 101);
    }

    #[test]
    fn default_starts_at_one() {
        let ids = IdGen::default();
        assert_eq!(ids.next(), 1);
    }

    #[test]
    fn ensure_at_least_never_moves_backwards() {
        let ids = IdGen::starting_at(5);
        ids.ensure_at_least(100);
        assert_eq!(ids.next(), 100);
        ids.ensure_at_least(10);
        assert_eq!(ids.next(), 101);
    }
}
