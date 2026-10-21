//! Port of `modules/util/shellquote.go`.
//!
//! Escapes a string so a POSIX shell re-reads it as a single literal word.
//! Gitea needs this because it feeds user-supplied paths (repo names, hook
//! arguments) to `sh -c`.

use std::fmt::Write as _;

const TILDE_PREFIX: char = '~';
const COMMENT_PREFIX: char = '#';

/// Characters that force escaping: whitespace, shell metacharacters, and the
/// characters with special meaning inside double quotes.
const NEEDS_ESCAPE: &str = " \t\n|&;()<>${}[]*?!\"'`\\";

/// Characters that cannot be expressed inside double quotes at all.
const NEEDS_SINGLE_QUOTE: &str = "!\n";

fn contains_any(s: &str, set: &str) -> bool {
    s.chars().any(|c| set.contains(c))
}

fn double_quote_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '$' | '`' | '"' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

fn single_quote_escape(s: &str) -> String {
    s.replace('\'', r"'\''")
}

/// Collapses the redundant quote runs the single-quote escaper produces.
fn single_quote_coalesce(s: &str) -> String {
    s.replace(r"''\'", r"\'").replace(r"\'''", r"\'")
}

/// `util.ShellEscape` — quotes `to_escape` for a POSIX shell.
///
/// Preferences, in order: leave a string that needs no escaping alone; prefer
/// double quotes; fall back to single quotes when the string contains `!` or a
/// newline, since those cannot be safely represented inside double quotes.
pub fn shell_escape(to_escape: &str) -> String {
    if to_escape.is_empty() {
        return to_escape.to_string();
    }

    let bytes = to_escape.as_bytes();
    let mut start = 0usize;

    if bytes[0] == TILDE_PREFIX as u8 {
        // A leading `~` is left bare so the shell performs tilde expansion,
        // but only when the segment before the first `/` has nothing to escape.
        let idx = match to_escape.find('/') {
            Some(i) => i + 1,
            None => to_escape.len(),
        };
        if !contains_any(&to_escape[..idx], NEEDS_ESCAPE) {
            start = idx;
        }
    }

    let rest = &to_escape[start..];
    if !contains_any(rest, NEEDS_ESCAPE) && bytes[0] != COMMENT_PREFIX as u8 {
        return to_escape.to_string();
    }

    let mut sb = String::with_capacity(to_escape.len() + 2);
    sb.push_str(&to_escape[..start]);

    if contains_any(rest, NEEDS_SINGLE_QUOTE) {
        let mut sb2 = String::with_capacity(rest.len() + 2);
        sb2.push('\'');
        sb2.push_str(&single_quote_escape(rest));
        sb2.push('\'');
        let _ = write!(sb, "{}", single_quote_coalesce(&sb2));
        return sb;
    }

    sb.push('"');
    sb.push_str(&double_quote_escape(rest));
    sb.push('"');
    sb
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `TestShellEscape` in modules/util/shellquote_test.go.
    #[test]
    fn shell_escape_matches_upstream() {
        let cases: &[(&str, &str, &str)] = &[
            (
                "nothing to escape",
                "a/b/c/d",
                "a/b/c/d",
            ),
            (
                "prefixed tilde, normal",
                "~/src/go/gitea/gitea",
                "~/src/go/gitea/gitea",
            ),
            (
                "windows path with spaces",
                r"C:\Program Files\Gitea v1.13 - I like lots of spaces\gitea",
                r#""C:\\Program Files\\Gitea v1.13 - I like lots of spaces\\gitea""#,
            ),
            (
                "forward-slash windows path with spaces",
                "C:/Program Files/Gitea v1.13 - I like lots of spaces/gitea",
                r#""C:/Program Files/Gitea v1.13 - I like lots of spaces/gitea""#,
            ),
            (
                "tilde then space-filled path",
                "~git/Gitea v1.13/gitea",
                r#"~git/"Gitea v1.13/gitea""#,
            ),
            (
                "bang forces single quotes",
                "C:/Program Files/Gitea!/gitea",
                "'C:/Program Files/Gitea!/gitea'",
            ),
            (
                "newlines",
                "/home/git/Gitea\n\nWHY-WOULD-YOU-DO-THIS\n\nGitea/gitea",
                "'/home/git/Gitea\n\nWHY-WOULD-YOU-DO-THIS\n\nGitea/gitea'",
            ),
            (
                "multiple single quotes while single-quoting",
                "'!''!'''!''!'!'",
                r"\''!'\'\''!'\'\'\''!'\'\''!'\''!'\'",
            ),
            ("less-than", "~/<gitea", "~/\"<gitea\""),
            ("greater-than", "~/gitea>", "~/\"gitea>\""),
            ("dollar", "~/$gitea", "~/\"\\$gitea\""),
            ("brace open", "~/{gitea", "~/\"{gitea\""),
            ("brace close", "~/gitea}", "~/\"gitea}\""),
            ("parens", "~/(gitea)", "~/\"(gitea)\""),
            ("backtick", "~/gitea`", "~/\"gitea\\`\""),
            ("leading comment char", "#123", "\"#123\""),
            (
                "mixed doubles",
                "~/<gitea> ${gitea} `gitea` [gitea] (gitea) \"gitea\" \\gitea\\ 'gitea'",
                "~/\"<gitea> \\${gitea} \\`gitea\\` [gitea] (gitea) \\\"gitea\\\" \\\\gitea\\\\ 'gitea'\"",
            ),
            (
                "single quotes only escape the quote itself",
                "~/<gitea> ${gitea} `gitea` (gitea) !gitea! \"gitea\" \\gitea\\ 'gitea'",
                "~/'<gitea> ${gitea} `gitea` (gitea) !gitea! \"gitea\" \\gitea\\ '\\''gitea'\\'",
            ),
        ];

        for (name, input, want) in cases {
            assert_eq!(&shell_escape(input), want, "case: {name}");
        }
    }

    #[test]
    fn empty_string_is_unchanged() {
        assert_eq!(shell_escape(""), "");
    }
}
