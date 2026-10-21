//! Port of `modules/util/trie.go`.

/// A byte-sequence prefix trie, used for fast prefix matching.
///
/// Mirrors upstream's slice-of-edges layout rather than a `HashMap`, because
/// the child list is small in practice and the scan order is part of the
/// observable behaviour for equal-length alternatives.
#[derive(Debug, Default)]
pub struct TrieNode {
    children: Vec<(u8, TrieNode)>,
    is_end: bool,
}

impl TrieNode {
    pub fn new() -> Self {
        Self::default()
    }

    fn child(&self, b: u8) -> Option<&TrieNode> {
        self.children.iter().find(|(c, _)| *c == b).map(|(_, n)| n)
    }

    fn child_mut(&mut self, b: u8) -> Option<&mut TrieNode> {
        self.children
            .iter_mut()
            .find(|(c, _)| *c == b)
            .map(|(_, n)| n)
    }

    /// `TrieNode.Insert` — adds a byte sequence to the trie.
    pub fn insert(&mut self, val: &str) {
        let mut curr = self;
        for b in val.bytes() {
            if curr.child(b).is_none() {
                curr.children.push((b, TrieNode::new()));
            }
            curr = curr.child_mut(b).expect("just inserted");
        }
        curr.is_end = true;
    }

    /// `TrieNode.Match` — length of the longest inserted prefix starting at
    /// `start`, or `-1` when nothing matches.
    pub fn match_at(&self, s: &str, start: usize) -> i32 {
        let bytes = s.as_bytes();
        let mut curr = self;
        let mut match_len: i32 = -1;
        for (j, &b) in bytes.iter().enumerate().skip(start) {
            match curr.child(b) {
                Some(next) => curr = next,
                None => break,
            }
            if curr.is_end {
                match_len = (j - start + 1) as i32;
            }
        }
        match_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `TestTrie` in modules/util/trie_test.go.
    #[test]
    fn trie_matches_upstream() {
        let mut trie = TrieNode::new();
        trie.insert("apple");
        trie.insert("apricot");
        trie.insert("banana");
        trie.insert("app");

        assert_eq!(trie.match_at("apple", 0), 5);
        assert_eq!(trie.match_at("apricot", 0), 7);
        assert_eq!(trie.match_at("banana", 0), 6);
        assert_eq!(trie.match_at("app", 0), 3);

        // Longest match wins over a shorter inserted prefix.
        assert_eq!(trie.match_at("apple-pie", 0), 5);

        // Matching can start mid-string.
        assert_eq!(trie.match_at("sweet apple", 6), 5);

        assert_eq!(trie.match_at("orange", 0), -1);
        // "ap" is a real prefix but was never inserted as a complete value.
        assert_eq!(trie.match_at("ap", 0), -1);
    }
}
