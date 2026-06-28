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
