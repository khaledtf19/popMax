/// Search shortcuts — type `!g query` to search Google, `!y query` for YouTube, etc.
///
/// The bang prefix must appear at the start of the input (leading whitespace is trimmed).

pub struct Bang {
    pub prefix: &'static str,
    pub name: &'static str,
    pub url_template: &'static str,
}

pub const BANGS: &[Bang] = &[
    Bang {
        prefix: "!g",
        name: "Google",
        url_template: "https://google.com/search?q={}",
    },
    Bang {
        prefix: "!y",
        name: "YouTube",
        url_template: "https://youtube.com/results?search_query={}",
    },
    Bang {
        prefix: "!w",
        name: "Wikipedia",
        url_template: "https://en.wikipedia.org/wiki/Special:Search?search={}",
    },
    Bang {
        prefix: "!gh",
        name: "GitHub",
        url_template: "https://github.com/search?q={}",
    },
    Bang {
        prefix: "!r",
        name: "Reddit",
        url_template: "https://www.reddit.com/search/?q={}",
    },
    Bang {
        prefix: "!a",
        name: "Amazon",
        url_template: "https://www.amazon.com/s?k={}",
    },
    Bang {
        prefix: "!s",
        name: "Stack Overflow",
        url_template: "https://stackoverflow.com/search?q={}",
    },
    Bang {
        prefix: "!m",
        name: "MDN",
        url_template: "https://developer.mozilla.org/en-US/search?q={}",
    },
    Bang {
        prefix: "!x",
        name: "X / Twitter",
        url_template: "https://x.com/search?q={}",
    },
];

/// Returns the matching `Bang` and the query string, or `None` if the input
/// doesn't start with any known bang prefix.
pub fn parse_bang(input: &str) -> Option<(&'static Bang, &str)> {
    let input = input.trim();
    for bang in BANGS {
        if let Some(rest) = input.strip_prefix(bang.prefix) {
            let query = rest.trim();
            if !query.is_empty() {
                return Some((bang, query));
            }
        }
    }
    None
}

/// Build the full search URL by inserting the URL-encoded query into the
/// bang's template.
pub fn search_url(bang: &Bang, query: &str) -> String {
    let encoded = urlencoding::encode(query);
    bang.url_template.replace("{}", &encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_google_bang() {
        let (bang, query) = parse_bang("!g hello world").expect("should parse !g");
        assert_eq!(bang.prefix, "!g");
        assert_eq!(bang.name, "Google");
        assert_eq!(query, "hello world");
    }

    #[test]
    fn parse_youtube_bang() {
        let (bang, query) = parse_bang("!y test video").expect("should parse !y");
        assert_eq!(bang.prefix, "!y");
        assert_eq!(query, "test video");
    }

    #[test]
    fn parse_bang_with_leading_spaces() {
        let (bang, query) = parse_bang("  !g query").expect("should parse with leading whitespace");
        assert_eq!(bang.prefix, "!g");
        assert_eq!(query, "query");
    }

    #[test]
    fn parse_bang_empty_query_returns_none() {
        assert!(parse_bang("!g").is_none(), "bare !g should return None");
        assert!(
            parse_bang("!g  ").is_none(),
            "!g with only spaces should return None"
        );
    }

    #[test]
    fn parse_bang_unknown_prefix_returns_none() {
        assert!(parse_bang("!z foo").is_none());
        assert!(parse_bang("!!bar").is_none());
    }

    #[test]
    fn parse_bang_no_bang_returns_none() {
        assert!(parse_bang("hello world").is_none());
    }

    #[test]
    fn search_url_encodes_spaces() {
        let (bang, _) = parse_bang("!g hello world").unwrap();
        let url = search_url(bang, "hello world");
        assert_eq!(url, "https://google.com/search?q=hello%20world");
    }

    #[test]
    fn search_url_encodes_special_chars() {
        let (bang, _) = parse_bang("!g a&b=c").unwrap();
        let url = search_url(bang, "a&b=c");
        assert_eq!(url, "https://google.com/search?q=a%26b%3Dc");
    }

    #[test]
    fn all_bangs_have_unique_prefixes() {
        let mut prefixes = std::collections::HashSet::new();
        for bang in BANGS {
            assert!(
                prefixes.insert(bang.prefix),
                "duplicate prefix: {}",
                bang.prefix
            );
        }
    }

    #[test]
    fn all_bangs_produce_valid_urls() {
        for bang in BANGS {
            let url = search_url(bang, "test");
            assert!(
                url.contains("test"),
                "bang {} did not insert query into URL",
                bang.prefix
            );
            assert!(
                url.starts_with("https://"),
                "bang {} URL does not start with https",
                bang.prefix
            );
        }
    }
}
