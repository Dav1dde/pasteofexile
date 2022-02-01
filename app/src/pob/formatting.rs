#[derive(Debug, PartialEq, Eq)]
pub enum Color<'a> {
    Hex(&'a str),
    Named(u8),
    None,
}

pub struct ColoredText<'a> {
    text: Option<&'a str>,
    style: Color<'a>,
}

impl<'a> ColoredText<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text: Some(text),
            style: Color::None,
        }
    }
}

impl<'a> Iterator for ColoredText<'a> {
    type Item = (Color<'a>, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let text = self.text.take()?;

        let mut start_search = 0;

        let (end, next_start, next_style) = loop {
            let index = match text[start_search..].find('^') {
                Some(index) => index,
                None => {
                    if text.is_empty() {
                        return None;
                    }
                    return Some((std::mem::replace(&mut self.style, Color::None), text));
                }
            };

            let r = match text.get(index + 1..index + 2) {
                Some("x") => text
                    .get(index + 2..index + 8)
                    .filter(|hex| hex.as_bytes().iter().all(u8::is_ascii_hexdigit))
                    .map(Color::Hex)
                    .map(|color| (8, color)),
                Some("0") => Some((2, Color::Named(0))),
                Some("1") => Some((2, Color::Named(1))),
                Some("2") => Some((2, Color::Named(2))),
                Some("3") => Some((2, Color::Named(3))),
                Some("4") => Some((2, Color::Named(4))),
                Some("5") => Some((2, Color::Named(5))),
                Some("6") => Some((2, Color::Named(6))),
                Some("7") => Some((2, Color::Named(7))),
                Some("8") => Some((2, Color::Named(8))),
                Some("9") => Some((2, Color::Named(9))),
                _ => None,
            };

            if let Some((offset, next_style)) = r {
                break (index, index + offset, next_style);
            }

            start_search = start_search + index + 1;
        };

        let style = std::mem::replace(&mut self.style, next_style);
        let result = (style, &text[..end]);

        self.text = text.get(next_start..);

        if result.1.is_empty() {
            // e.g. "^1foo" produces an empty range
            // or "^1^2foo" produces two empty ranges
            self.next()
        } else {
            Some(result)
        }
    }
}

pub fn strip_colors(text: &str) -> String {
    ColoredText::new(text).map(|(_, text)| text).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use Color::*;

    #[test]
    fn test_no_element() {
        let x = ColoredText::new("Test").collect::<Vec<_>>();
        assert_eq!(x, vec![(None, "Test")]);
    }

    #[test]
    fn test_one_hex_element() {
        let x = ColoredText::new("^x001122Test").collect::<Vec<_>>();
        assert_eq!(x, vec![(Hex("001122"), "Test")]);
    }

    #[test]
    fn test_one_named_element() {
        let x = ColoredText::new("^2Test").collect::<Vec<_>>();
        assert_eq!(x, vec![(Named(2), "Test")]);
    }

    #[test]
    fn test_invalid_hex() {
        let x = ColoredText::new("^x00112ZTest").collect::<Vec<_>>();
        assert_eq!(x, vec![(None, "^x00112ZTest")]);
    }

    #[test]
    fn test_too_short() {
        let x = ColoredText::new("Test^").collect::<Vec<_>>();
        assert_eq!(x, vec![(None, "Test^")]);
    }

    #[test]
    fn test_too_short_hex() {
        let x = ColoredText::new("Test^x00112").collect::<Vec<_>>();
        assert_eq!(x, vec![(None, "Test^x00112")]);
    }

    #[test]
    fn test_multiple() {
        let x = ColoredText::new("First^x001122Hex^3Num").collect::<Vec<_>>();
        assert_eq!(
            x,
            vec![(None, "First"), (Hex("001122"), "Hex"), (Named(3), "Num")]
        );
    }

    #[test]
    fn test_multiple_empty() {
        let x = ColoredText::new("^1^2^3^4^5").collect::<Vec<_>>();
        assert_eq!(x, vec![]);
    }

    #[test]
    fn test_strip_colors() {
        let x = strip_colors("foo^1bar^x001122 baz^brokenx");
        assert_eq!(x, "foobar baz^brokenx");
    }
}
