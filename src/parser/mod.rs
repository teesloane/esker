pub mod headlines;
pub mod links;
pub mod syntax_highlight;

use crate::{link::Link, md_file::MdFile, site::Site};
use pulldown_cmark::{html, Event, HeadingLevel, Options, Parser, Tag};
use slugify::slugify;
use syntax_highlight::CodeBlockSyntaxHighlight;

use self::headlines::ParseHeadlines;

pub fn new(parser: Parser, md_file: &mut MdFile, site: &mut Site) -> String {
    // -- parser stuff

    // TODO: I don't know how to abstract this into another function with correct lifetimes.
    let mut capturing = false;
    let mut capturing_heading = false;
    let mut link = Link::empty();
    let mut toc_link_placeholder = Link::empty();

    let parser = parser.map(|event| -> Event {
        match event {
            Event::Start(tag) => match tag {
                Tag::Link(link_type, ref url, ref title) => {
                    link.update_vals(
                        tag,
                        site,
                        Some(md_file.full_url.clone()),
                        Some(md_file.frontmatter.title.clone()),
                    );
                    capturing = true;
                    Event::Start(link.for_parser(site))
                }
                Tag::Image(link_type, url, title) => {
                    Event::Start(Link::update_img_link(link_type, url, title, site))
                }

                Tag::Heading(heading_level, fragment_id, css_classes) => {
                    capturing_heading = true;
                    Event::Start(Tag::Heading(heading_level, fragment_id, css_classes))
                }

                _ => Event::Start(tag),
            },

            Event::Text(text) => {
                if capturing {
                    link.title = text.to_string();
                    capturing = false
                }

                if capturing_heading {
                    toc_link_placeholder.title = format!("{}{}", toc_link_placeholder.title, text);
                    toc_link_placeholder.url = format!(
                        "{}#{}",
                        md_file.full_url,
                        slugify!(&toc_link_placeholder.title.to_string())
                    );
                }
                Event::Text(text)
            }

            Event::End(tag) => match tag {
                Tag::Link(link_type, url, title) => {
                    site.add_link(link.clone());
                    Event::End(Tag::Link(link_type, url, title))
                }

                Tag::Heading(a, b, c) => {
                    capturing_heading = false;
                    md_file.toc.push(toc_link_placeholder.clone());
                    toc_link_placeholder = Link::empty();
                    Event::End(Tag::Heading(a, b, c))
                }
                _ => Event::End(tag),
            },

            _ => event,
        }
    });

    // transformation section
    let parser = CodeBlockSyntaxHighlight::new(parser);
    let parser = ParseHeadlines::new(parser);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
