//! Port of `modules/util/slice.go`.

/// `util.SliceContainsString` — membership test, optionally case-insensitive.
pub fn slice_contains_string(slice: &[String], target: &str, insensitive: bool) -> bool {
    if insensitive {
        slice.iter().any(|t| t.eq_ignore_ascii_case(target))
    } else {
        slice.iter().any(|t| t == target)
    }
}

/// `util.SliceSortedEqual` — true when both slices hold the same multiset.
pub fn slice_sorted_equal<T: PartialEq + Eq + std::hash::Hash>(s1: &[T], s2: &[T]) -> bool {
    if s1.len() != s2.len() {
        return false;
    }
    let mut counts: std::collections::HashMap<&T, isize> = std::collections::HashMap::new();
    for v in s1 {
        *counts.entry(v).or_insert(0) += 1;
    }
    for v in s2 {
        *counts.entry(v).or_insert(0) -= 1;
    }
    counts.values().all(|c| *c == 0)
}

/// `util.SliceRemoveAll` — removes every occurrence of `target`.
pub fn slice_remove_all<T: PartialEq>(slice: &mut Vec<T>, target: &T) {
    slice.retain(|t| t != target);
}

/// `util.SliceNilAsEmpty` — Go's nil-slice-to-empty-slice normalisation.
///
/// In Rust this only matters for serialisation, where an empty `Vec` must not
/// become JSON `null`; the identity function documents that contract.
pub fn slice_nil_as_empty<T>(a: Vec<T>) -> Vec<T> {
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_sorted_equal_ignores_order() {
        assert!(slice_sorted_equal(&[1, 2, 3], &[3, 2, 1]));
        assert!(slice_sorted_equal(&[1, 1, 2], &[1, 2, 1]));
        assert!(!slice_sorted_equal(&[1, 1, 2], &[1, 2, 2]));
        assert!(!slice_sorted_equal(&[1, 2], &[1, 2, 3]));
    }

    #[test]
    fn slice_contains_string_honours_insensitive_flag() {
        let v = vec!["Foo".to_string()];
        assert!(slice_contains_string(&v, "Foo", false));
        assert!(!slice_contains_string(&v, "foo", false));
        assert!(slice_contains_string(&v, "foo", true));
    }

    #[test]
    fn slice_remove_all_removes_every_occurrence() {
        let mut v = vec![1, 2, 1, 3, 1];
        slice_remove_all(&mut v, &1);
        assert_eq!(v, [2, 3]);
    }
}
