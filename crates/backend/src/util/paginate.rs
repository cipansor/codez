//! Port of `modules/util/paginate.go`.

/// `util.PaginateSlice` — slices a list for a 1-based page.
///
/// A non-positive `page` or `page_size` means "no pagination" and returns the
/// list unchanged, matching upstream.
pub fn paginate_slice<T: Clone>(list: &[T], page: i64, page_size: i64) -> Vec<T> {
    if page <= 0 || page_size <= 0 {
        return list.to_vec();
    }

    let start = (page - 1) * page_size;
    if start >= list.len() as i64 {
        return Vec::new();
    }

    let start = start as usize;
    let end = (start + page_size as usize).min(list.len());
    list[start..end].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_positive_page_or_size_returns_everything() {
        let list = vec![1, 2, 3, 4, 5];
        assert_eq!(paginate_slice(&list, 0, 2), list);
        assert_eq!(paginate_slice(&list, 1, 0), list);
        assert_eq!(paginate_slice(&list, -1, -1), list);
    }

    #[test]
    fn paginates_one_based() {
        let list = vec![1, 2, 3, 4, 5];
        assert_eq!(paginate_slice(&list, 1, 2), [1, 2]);
        assert_eq!(paginate_slice(&list, 2, 2), [3, 4]);
        assert_eq!(paginate_slice(&list, 3, 2), [5]);
        assert_eq!(paginate_slice(&list, 4, 2), Vec::<i32>::new());
    }
}
