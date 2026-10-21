//! Port of `modules/container/filter.go`.

/// `container.FilterSlice` — maps each element with `include` and keeps the
/// values whose flag is true, dropping duplicates.
///
/// `T` is `Hash + Eq + Clone` because upstream deduplicates through a map; the
/// first occurrence of a value wins, and the survivors keep the input order.
pub fn filter_slice<E, T: std::hash::Hash + Eq + Clone>(
    s: &[E],
    include: impl Fn(&E) -> (T, bool),
) -> Vec<T> {
    let mut filtered: Vec<T> = Vec::with_capacity(s.len());
    let mut seen: std::collections::HashSet<T> = std::collections::HashSet::with_capacity(s.len());
    for e in s {
        let (v, ok) = include(e);
        if ok && seen.insert(v.clone()) {
            filtered.push(v);
        }
    }
    filtered
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `TestFilterMapUnique` in modules/container/filter_test.go.
    #[test]
    fn filter_slice_matches_upstream() {
        let result = filter_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], |i| match i {
            0 => (0, true),  // included later
            1 => (0, true),  // duplicate of the previous, ignored
            2 => (2, false), // not included
            _ => (*i, true),
        });
        assert_eq!(result, vec![0, 3, 4, 5, 6, 7, 8, 9]);
    }
}
