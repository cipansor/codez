//! Port of `gitea.dev/modules/regexplru` — bounded LRU of compiled regexps.
//!
//! | upstream | here |
//! | --- | --- |
//! | `regexplru.go` | this module |

use regex::Regex;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};

/// One cached entry: the compiled regexp or the compile error.
#[derive(Clone)]
struct LruItem {
    regexp: Option<Arc<Regex>>,
    err: Option<String>,
}

/// A bounded least-recently-used cache of compiled regexps.
pub struct RegexpCache {
    size: usize,
    inner: Mutex<CacheInner>,
}

struct CacheInner {
    map: HashMap<String, LruItem>,
    /// Recency order: front is least recently used, back is most recent.
    order: VecDeque<String>,
}

impl RegexpCache {
    /// `regexplru.NewCache`.
    pub fn new(size: usize) -> Self {
        Self {
            size,
            inner: Mutex::new(CacheInner {
                map: HashMap::new(),
                order: VecDeque::new(),
            }),
        }
    }

    /// `RegexpCache.GetCompiled` — like `Regex::new`, but the compiled program
    /// or its error is memoised.
    ///
    /// Both hits and misses are cached, so a repeatedly-invalid pattern is not
    /// recompiled either.
    pub fn get_compiled(&self, expr: &str) -> Result<Arc<Regex>, String> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());

        if let Some(item) = inner.map.get(expr) {
            let item = item.clone();
            touch(&mut inner.order, expr);
            return match item.regexp {
                Some(r) => Ok(r),
                None => Err(item.err.unwrap_or_else(|| "compile failed".to_string())),
            };
        }

        let item = match Regex::new(expr) {
            Ok(r) => LruItem {
                regexp: Some(Arc::new(r)),
                err: None,
            },
            Err(e) => LruItem {
                regexp: None,
                err: Some(e.to_string()),
            },
        };
        let result = match &item.regexp {
            Some(r) => Ok(Arc::clone(r)),
            None => Err(item
                .err
                .clone()
                .unwrap_or_else(|| "compile failed".to_string())),
        };

        inner.map.insert(expr.to_string(), item);
        inner.order.push_back(expr.to_string());
        evict(&mut inner, self.size);
        result
    }

    /// Number of cached entries; mirrors upstream's `lruCache.Len()`.
    pub fn len(&self) -> usize {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .map
            .len()
    }

    /// Whether the cache holds no entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Moves `key` to the most-recently-used position.
fn touch(order: &mut VecDeque<String>, key: &str) {
    if let Some(pos) = order.iter().position(|k| k == key) {
        if let Some(k) = order.remove(pos) {
            order.push_back(k);
        }
    }
}

/// Drops entries until the cache is within `size`.
fn evict(inner: &mut CacheInner, size: usize) {
    while inner.map.len() > size {
        match inner.order.pop_front() {
            Some(oldest) => {
                inner.map.remove(&oldest);
            }
            None => break,
        }
    }
}

/// `regexplru.UserCache` — the shared cache used for user-supplied patterns.
pub fn user_cache() -> &'static RegexpCache {
    static CACHE: OnceLock<RegexpCache> = OnceLock::new();
    CACHE.get_or_init(|| RegexpCache::new(1000))
}

/// `regexplru.SystemCache` — the shared cache for patterns Codez itself defines.
pub fn system_cache() -> &'static RegexpCache {
    static CACHE: OnceLock<RegexpCache> = OnceLock::new();
    CACHE.get_or_init(|| RegexpCache::new(1000))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `TestRegexpLru` in modules/regexplru/regexplru_test.go.
    #[test]
    fn regexp_lru_matches_upstream() {
        // A private cache keeps this test independent of the shared ones.
        let cache = RegexpCache::new(1000);

        let r = cache.get_compiled("a").unwrap();
        assert!(r.is_match("a"));

        let r = cache.get_compiled("a").unwrap();
        assert!(r.is_match("a"));
        assert_eq!(cache.len(), 1, "a repeat lookup must not add an entry");

        assert!(cache.get_compiled("(").is_err());
        assert_eq!(cache.len(), 2, "a compile error is cached too");
    }

    #[test]
    fn shared_caches_are_reused() {
        assert!(std::ptr::eq(user_cache(), user_cache()));
        assert!(std::ptr::eq(system_cache(), system_cache()));
    }

    #[test]
    fn the_cache_evicts_the_least_recently_used_entry() {
        let cache = RegexpCache::new(2);
        cache.get_compiled("a").unwrap();
        cache.get_compiled("b").unwrap();
        // Touch "a" so "b" becomes the least recently used.
        cache.get_compiled("a").unwrap();
        cache.get_compiled("c").unwrap();

        assert_eq!(cache.len(), 2);
        let inner = cache.inner.lock().unwrap();
        assert!(inner.map.contains_key("a"));
        assert!(!inner.map.contains_key("b"));
        assert!(inner.map.contains_key("c"));
    }
}
