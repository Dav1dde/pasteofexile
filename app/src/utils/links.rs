pub struct Links<'a> {
    text: Option<&'a str>,
    next: Option<&'a str>,
}

impl<'a> Links<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text: Some(text),
            next: None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Link<'a> {
    Link(&'a str),
    Text(&'a str),
}

impl<'a> Iterator for Links<'a> {
    type Item = Link<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(link) = self.next.take() {
            return Some(Link::Link(link));
        }

        let text = self.text.take()?;
        if text.is_empty() {
            return None;
        }

        let mut start_search = 0;
        let (index, link) = loop {
            const PROTOCOL: &str = "https://";

            let index = match text.get(start_search..).and_then(|s| s.find(PROTOCOL)) {
                Some(index) => start_search + index,
                None => return Some(Link::Text(text)),
            };

            let link = {
                let text = &text[index..];
                let link_end = text.find(is_link_end).unwrap_or(text.len());
                &text[..link_end]
            };

            let domain = {
                let domain_end = link[PROTOCOL.len()..]
                    .find('/')
                    .unwrap_or(link.len() - PROTOCOL.len());
                &link[PROTOCOL.len()..PROTOCOL.len() + domain_end]
            };

            if !is_domain_whitelisted(domain) {
                start_search += PROTOCOL.len();
                continue;
            }

            break (index, link);
        };

        if index == 0 {
            self.text = Some(&text[link.len()..]);
            Some(Link::Link(link))
        } else {
            self.next = Some(link);
            self.text = Some(&text[index + link.len()..]);

            Some(Link::Text(&text[..index]))
        }
    }
}

fn is_link_end(c: char) -> bool {
    c.is_whitespace() || matches!(c, ')' | ']' | '}')
}

fn is_domain_whitelisted(domain: &str) -> bool {
    let domain = domain.strip_prefix("www.").unwrap_or(domain);
    crate::consts::LINK_WHITELIST.contains(&domain)
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::Link::*;
    use super::*;

    #[test]
    pub fn link_only() {
        let r = Links::new("https://pobb.in/foo/bar?test=123").collect_vec();
        assert_eq!(r, vec![Link("https://pobb.in/foo/bar?test=123")]);
    }

    #[test]
    pub fn link_start() {
        let r = Links::new("https://pobb.in/foo/bar?test=123 more text").collect_vec();
        assert_eq!(
            r,
            vec![Link("https://pobb.in/foo/bar?test=123"), Text(" more text")]
        );
    }

    #[test]
    pub fn link_end() {
        let r = Links::new("some text https://pobb.in/foo/bar?test=123").collect_vec();
        assert_eq!(
            r,
            vec![Text("some text "), Link("https://pobb.in/foo/bar?test=123")]
        );
    }

    #[test]
    pub fn link_mid() {
        let r = Links::new("some text https://pobb.in/foo/bar?test=123 more text").collect_vec();
        assert_eq!(
            r,
            vec![
                Text("some text "),
                Link("https://pobb.in/foo/bar?test=123"),
                Text(" more text")
            ]
        );
    }

    #[test]
    pub fn link_no_path() {
        let r = Links::new("some text https://pobb.in asd").collect_vec();
        assert_eq!(
            r,
            vec![Text("some text "), Link("https://pobb.in"), Text(" asd")]
        );
    }

    #[test]
    pub fn link_no_path_trailing_slash() {
        let r = Links::new("some text https://pobb.in/ asd").collect_vec();
        assert_eq!(
            r,
            vec![Text("some text "), Link("https://pobb.in/"), Text(" asd")]
        );
    }

    #[test]
    pub fn link_no_whitelist() {
        let r = Links::new("some text https://example.org/ asd").collect_vec();
        assert_eq!(r, vec![Text("some text https://example.org/ asd")]);
    }

    #[test]
    pub fn link_no_whitelist_mixed() {
        let r = Links::new("some text https://example.org/ https://pobb.in asd").collect_vec();
        assert_eq!(
            r,
            vec![
                Text("some text https://example.org/ "),
                Link("https://pobb.in"),
                Text(" asd")
            ]
        );
    }

    #[test]
    pub fn link_multiple() {
        let r = Links::new("some text https://pobb.in/abc https://pobb.in asd").collect_vec();
        assert_eq!(
            r,
            vec![
                Text("some text "),
                Link("https://pobb.in/abc"),
                Text(" "),
                Link("https://pobb.in"),
                Text(" asd")
            ]
        );
    }

    #[test]
    pub fn link_www_optional_prefix() {
        let r = Links::new("https://www.pobb.in").collect_vec();
        assert_eq!(r, vec![Link("https://www.pobb.in")]);
    }

    #[test]
    pub fn link_end_non_space() {
        let r = Links::new("(https://pobb.in)").collect_vec();
        assert_eq!(r, vec![Text("("), Link("https://pobb.in"), Text(")")]);
    }
}
