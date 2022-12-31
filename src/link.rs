use crate::site::Site;
use pulldown_cmark::{CowStr, LinkType, Tag};
use slugify::slugify;
use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug)]
pub struct SiteLinks {
    pub external: Vec<Link>,
    pub internal: Vec<Link>,
}

impl SiteLinks {
    pub fn new() -> SiteLinks {
        SiteLinks {
            external: Vec::new(),
            internal: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Link {
    pub url: String,
    pub is_internal: bool,
    pub title: String,
    pub originating_file_title: String,
    pub originating_file_url: String,
}

// TODO: this could probably be grouped in a file with the parser for better organizations.
// Or, just moved into md_file

// NOTE: find a way to extract update link and update_img_link into one
// function. FUTURE ME TIP: I tried just passing the "tag" (Tag::Image/Link) and
// then matching on that. It worked almost perfectly except that I ran into
// borrow issues because of "partial borrows" ....
impl Link {
    pub fn update_vals(
        &mut self,
        tag: Tag,
        site: &mut Site,
        originating_url: String,
        originating_title: String,
    ) {
        match tag {
            Tag::Link(link_type, url, title) => {
                let mut url_str = Self::slugify_internal_url(url.to_string().clone());
                if Self::is_internal(&url) {
                    let url_as_path = PathBuf::from(&url_str).with_extension("html");
                    url_str = format!("{}", url_as_path.display());

                    let mut new_link_url: CowStr;
                    if Self::is_mailto(&url.to_string().clone()) {
                        new_link_url = url
                    } else {
                        new_link_url = site.build_with_baseurl(url_str).into();
                    }

                    self.url = new_link_url.to_string();
                    self.is_internal = true;
                } else {
                    self.url = url.to_string();
                    self.is_internal = false;
                }

                self.title = title.to_string();
                self.originating_file_title = originating_title;
                self.originating_file_url = originating_url
            }
            _ => panic!(),
        }
    }

    pub fn empty() -> Link {
        Link {
            url: String::from(""),
            is_internal: false,
            title: String::from(""),
            originating_file_url: String::from(""),
            originating_file_title: String::from(""),
        }
    }

    // takes a text link and updates it to add the base url if it's internal.
    pub fn for_parser<'a>(&self, site: &mut Site) -> Tag<'a> {
        let new_link_url: CowStr = self.url.clone().into();
        let title: CowStr = site.build_with_baseurl(self.url.clone()).into();
        return Tag::Link(LinkType::Inline, new_link_url, title);
    }

    // split a url: "projects/my_folder/a file"
    // get the last and slug it and rebuild the url.
    fn slugify_internal_url(url: String) -> String {
        let chunks: Vec<_> = url.split("/").collect();
        let slug_chunks: Vec<String> = chunks
            .iter()
            .map(|s| {
                let url_as_path = PathBuf::from(s).with_extension("");
                let url_as_string = url_as_path.into_os_string().into_string().unwrap();

                // replace all `%20` with `-`
                let new_str = url_as_string.replace("%20", "-");
                return slugify!(&new_str);
            })
            .collect();
        let rebuild_url = slug_chunks.join("/");
        return rebuild_url;
    }

    pub fn update_img_link<'a>(
        link_type: LinkType,
        url: CowStr<'a>,
        title: CowStr<'a>,
        site: &mut Site,
    ) -> Tag<'a> {
        let mut url_str = url.to_string();
        if Self::is_internal(&url_str) {
            let url_as_path = PathBuf::from(&url_str);
            url_str = format!("{}", url_as_path.display());
            let new_link_url: CowStr = site.build_with_baseurl(url_str).into();
            return Tag::Image(link_type, new_link_url, title);
        } else {
            return Tag::Image(link_type, url, title);
        }
    }

    pub fn is_internal(url: &str) -> bool {
        return !(Self::is_external(url));
    }

    pub fn is_mailto(url: &str) -> bool {
        return url.starts_with("mailto:");
    }

    pub fn is_external(url: &str) -> bool {
        return url.starts_with("http://")
            || url.starts_with("https://")
            || url.starts_with("www.");
    }

    pub fn is_attachment() -> bool {
        return false;
    }
}
