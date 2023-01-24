pub mod headlines;
pub mod links;
pub mod syntax_highlight;

use crate::{link::{Link, EskerLinkType}, md_file::MdFile, site::Site};
use pulldown_cmark::{html, Event, Parser, Tag, Options};
use slugify::slugify;
use syntax_highlight::CodeBlockSyntaxHighlight;

use self::headlines::ParseHeadlines;

pub fn new(md_file: &mut MdFile, site: &mut Site) -> String {
    // TODO: how can I not clone this here?
    let raw = md_file.raw.clone();
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    let mut parser = Parser::new_ext(&raw, options);

    // -- parser stuff

    // TODO: I don't know how to abstract this into another function with correct lifetimes.
    let mut capturing = false;
    let mut capturing_heading = false;
    let mut link = Link::empty();
    let mut toc_link_placeholder = Link::empty();

    let parser = parser.map(|event| -> Event {
        match event {
            Event::Start(tag) => match tag {
                Tag::Link(_link_type, ref _url, ref _title) => {
                    link.fill_from_parser(
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

                Tag::Heading(level, b, c) => {
                    capturing_heading = false;
                    toc_link_placeholder.link_type = EskerLinkType::Toc { heading_level: level as u8 };
                    md_file.toc.push(toc_link_placeholder.clone());
                    toc_link_placeholder = Link::empty();
                    Event::End(Tag::Heading(level, b, c))
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
