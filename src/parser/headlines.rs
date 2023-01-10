use std::fmt::Write;

use pulldown_cmark::{html::push_html, Event, Tag};
use slugify::slugify;

pub struct ParseHeadlines<'a, I: Iterator<Item = Event<'a>>> {
    parent: I,
}

impl<'a, I: Iterator<Item = Event<'a>>> ParseHeadlines<'a, I> {
    pub fn new(parent: I) -> Self {
        Self { parent }
    }
}

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for ParseHeadlines<'a, I> {
    type Item = Event<'a>;

    /// replace headlines with html headlines, such that they are given id's based on their child text.
    fn next(&mut self) -> Option<Self::Item> {
        let (heading_level, id_fragment, c) = match self.parent.next()? {
            Event::Start(pulldown_cmark::Tag::Heading(heading_level, id_fragment, c)) => {
                (heading_level, id_fragment, c)
            }
            other => return Some(other),
        };

        let mut events = Vec::new();
        let mut generated_id = String::new();

        loop {
            match self.parent.next()? {
                Event::End(Tag::Heading(_, _, _)) => break,
                Event::Text(text) => {
                    events.push(Event::Text(text.clone()));
                    generated_id.push_str(&format!(" {}", text));
                }
                event => events.push(event),
            }
        }

        let id = match id_fragment {
            Some(id) => id.to_string(),
            None => slugify!(&generated_id).to_string(),
        };

        let mut inner_html = String::new();
        push_html(&mut inner_html, events.iter().cloned());

        let mut res = String::new();

        write!(res, r#"<{heading_level} id="{id}">"#).unwrap();
        res.push_str(&inner_html);
        write!(res, "</{heading_level}>").unwrap();

        Some(Event::Html(res.into()))
    }
}
