//! Port of `modules/container/set.go`.
//!
//! Upstream's `Set[T]` is a named map type; the methods are a thin convenience
//! layer over Go's built-in map operations. `std::collections::HashSet` already
//! provides insert/contains/remove, so this type adds only the Gitea-specific
//! conveniences (`SetOf`, `AddMultiple`, `RemoveFromSlice`, `Union`) that keep
//! call sites readable.

use std::collections::HashSet;
use std::hash::Hash;

/// `container.Set[T]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Set<T: Eq + Hash> {
    inner: HashSet<T>,
}

// Hand-written so an empty set needs no `T: Default` bound, which a derive
// would add.
impl<T: Eq + Hash> Default for Set<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash> Set<T> {
    /// `container.SetOf` — builds a set from the given elements.
    pub fn of(values: impl IntoIterator<Item = T>) -> Self {
        Self {
            inner: values.into_iter().collect(),
        }
    }

    /// Creates an empty set.
    pub fn new() -> Self {
        Self {
            inner: HashSet::new(),
        }
    }

    /// `Set.Add` — `true` when the element was not already present.
    pub fn add(&mut self, value: T) -> bool {
        self.inner.insert(value)
    }

    /// `Set.AddMultiple` — adds every element.
    pub fn add_multiple(&mut self, values: impl IntoIterator<Item = T>) {
        self.inner.extend(values);
    }

    /// `Set.Contains` for a single borrowed element.
    pub fn has(&self, value: &T) -> bool {
        self.inner.contains(value)
    }

    /// `Set.Contains` — `true` only when *all* the given elements are present.
    ///
    /// An empty argument list is vacuously true, matching upstream's
    /// `ret := true` initialisation.
    pub fn contains(&self, values: impl IntoIterator<Item = T>) -> bool {
        values.into_iter().all(|v| self.inner.contains(&v))
    }

    /// `Set.Remove` — `true` when the element was present and removed.
    pub fn remove(&mut self, value: &T) -> bool {
        self.inner.remove(value)
    }

    /// `Set.RemoveFromSet` — number of elements successfully removed.
    pub fn remove_from_set(&mut self, other: &Set<T>) -> usize {
        other.inner.iter().filter(|v| self.inner.remove(*v)).count()
    }

    /// `Set.RemoveFromSlice` — number of elements successfully removed.
    pub fn remove_from_slice(&mut self, other: &[T]) -> usize {
        other.iter().filter(|v| self.inner.remove(*v)).count()
    }

    /// `Set.Values` — the elements in arbitrary order.
    pub fn values(&self) -> Vec<&T> {
        self.inner.iter().collect()
    }

    /// Number of elements, mirroring Go's `len(s)`.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<T: Eq + Hash + Clone> Set<T> {
    /// `Set.Union` — a new set containing this set and all `sets`.
    pub fn union(&self, sets: impl IntoIterator<Item = Set<T>>) -> Set<T> {
        let mut new_set = self.clone();
        for s in sets {
            new_set.inner.extend(s.inner);
        }
        new_set
    }
}

impl<T: Eq + Hash> FromIterator<T> for Set<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `TestSet` in modules/container/set_test.go.
    #[test]
    fn set_matches_upstream() {
        let mut s: Set<String> = Set::new();

        assert!(s.add("key1".to_string()));
        assert!(!s.add("key1".to_string()));
        assert!(s.add("key2".to_string()));

        assert!(s.contains(["key1".to_string()]));
        assert!(s.contains(["key2".to_string()]));
        assert!(s.contains(["key1".to_string(), "key2".to_string()]));
        assert!(!s.contains(["key3".to_string()]));
        assert!(!s.contains(["key1".to_string(), "key3".to_string()]));

        assert!(s.remove(&"key2".to_string()));
        assert!(!s.contains(["key2".to_string()]));

        assert!(!s.remove(&"key3".to_string()));

        s.add_multiple(["key4".to_string(), "key5".to_string()]);
        assert!(s.contains(["key4".to_string()]));
        assert!(s.contains(["key5".to_string()]));

        let s = Set::of(["key6".to_string(), "key7".to_string()]);
        assert!(!s.contains(["key1".to_string()]));
        assert!(s.contains(["key6".to_string()]));
        assert!(s.contains(["key7".to_string()]));

        let mut s = Set::of(["a".to_string(), "b".to_string(), "c".to_string()]);
        let n = s.remove_from_set(&Set::of([
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ]));
        assert_eq!(n, 2);
        let mut vals: Vec<&String> = s.values();
        vals.sort();
        assert_eq!(vals, vec!["a"]);

        let mut s = Set::of(["a".to_string(), "b".to_string(), "c".to_string()]);
        let n = s.remove_from_slice(&["b".to_string(), "c".to_string(), "d".to_string()]);
        assert_eq!(n, 2);
        let mut vals: Vec<&String> = s.values();
        vals.sort();
        assert_eq!(vals, vec!["a"]);
    }

    #[test]
    fn union_combines_sets() {
        let a = Set::of([1, 2]);
        let b = Set::of([2, 3]);
        let u = a.union([b]);
        let mut vals: Vec<&i32> = u.values();
        vals.sort();
        assert_eq!(vals, vec![&1, &2, &3]);
    }

    #[test]
    fn contains_with_no_values_is_true() {
        let s: Set<i32> = Set::new();
        assert!(s.contains([]));
    }
}
