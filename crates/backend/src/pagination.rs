//! Gitea-compatible pagination.
//!
//! Gitea reads `page` (1-based) and `limit` from the query string, and returns
//! `X-Total-Count` plus `X-HasMore` headers alongside the page body. The
//! defaults below match the upstream constants.

use serde::Deserialize;

/// Default page size when the client does not ask for one.
pub const DEFAULT_LIMIT: i64 = 20;

/// Hard cap on `limit`, mirroring Gitea's `setting.API.MaxResponseItems`.
pub const MAX_LIMIT: i64 = 50;

/// Query parameters understood by every list endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(DEFAULT_LIMIT),
        }
    }
}

impl Pagination {
    /// Normalised page number, never below 1.
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    /// Normalised page size, clamped to `1..=MAX_LIMIT`.
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
    }

    /// Zero-based index of the first item on this page.
    pub fn offset(&self) -> usize {
        ((self.page() - 1) * self.limit()) as usize
    }

    /// Slice `items` down to the requested page.
    pub fn apply<T>(&self, items: &[T]) -> Vec<T>
    where
        T: Clone,
    {
        let start = self.offset().min(items.len());
        let end = (start + self.limit() as usize).min(items.len());
        items[start..end].to_vec()
    }

    /// Total number of pages for `total` items.
    pub fn pages(&self, total: usize) -> i64 {
        let limit = self.limit();
        ((total as i64) + limit - 1) / limit
    }

    /// Whether another page exists after this one.
    pub fn has_more(&self, total: usize) -> bool {
        self.page() < self.pages(total)
    }

    /// `X-Total-Count` / `X-HasMore` style header pairs.
    pub fn headers(&self, total: usize) -> [(&'static str, String); 2] {
        [
            ("X-Total-Count", total.to_string()),
            ("X-HasMore", self.has_more(total).to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_gitea() {
        let p = Pagination::default();
        assert_eq!(p.page(), 1);
        assert_eq!(p.limit(), DEFAULT_LIMIT);
        assert_eq!(p.offset(), 0);
    }

    #[test]
    fn limit_is_clamped() {
        let p = Pagination {
            page: Some(0),
            limit: Some(10_000),
        };
        assert_eq!(p.page(), 1);
        assert_eq!(p.limit(), MAX_LIMIT);

        let zero = Pagination {
            page: None,
            limit: Some(0),
        };
        assert_eq!(zero.limit(), 1);
    }

    #[test]
    fn slices_pages() {
        let items: Vec<i32> = (1..=25).collect();
        let first = Pagination {
            page: Some(1),
            limit: Some(10),
        };
        assert_eq!(first.apply(&items), (1..=10).collect::<Vec<_>>());
        assert_eq!(first.pages(items.len()), 3);
        assert!(first.has_more(items.len()));

        let last = Pagination {
            page: Some(3),
            limit: Some(10),
        };
        assert_eq!(last.apply(&items), (21..=25).collect::<Vec<_>>());
        assert!(!last.has_more(items.len()));
    }

    #[test]
    fn past_the_end_yields_empty() {
        let p = Pagination {
            page: Some(99),
            limit: Some(10),
        };
        assert!(p.apply(&[1, 2, 3]).is_empty());
    }
}
