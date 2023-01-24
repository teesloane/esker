/// this file is responsible for providing structs and their requisite methods
/// that take internal data and prepare it for being inserted into a tera context.

use crate::{link::Link, md_file::MdFile, util, site::Site};
use serde::Serialize;
use std::path::Path;
use tera::Tera;

pub fn load_templates(dir_templates: &Path) -> Tera {
    let template_path = format!("{}/**/*.html", util::path_to_string(dir_templates));
    let mut tera = match Tera::new(&template_path) {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            util::exit();
        }
    };

    tera.add_template_file(dir_templates.join("feed.rss"), Some("feed.rss")).unwrap();

    tera.autoescape_on(vec![]);
    if tera.templates.is_empty() {
        println!("\nError: No templates found in {:?}\n", template_path);
        util::exit();
    }
    tera
}

// get_template returns the name of a template (to later render), provided it's
// found in the tera instance.
pub fn get_name(tera: &Tera, template: &str) -> String {
    let template_with_html = format!("{}.html", &template);
    let default_template_name = "single.html".to_string();
    if tera.get_template_names().any(|x| x == template_with_html) {
        return template_with_html;
    } else {
        return default_template_name;
    }
}

// Structs for tera
//
//
#[derive(Serialize)]
pub struct Config {
    title: String,
    description: String,
    url: String
}

impl Config{
    pub fn new(site: &Site) -> Self {
        Self {
            title: site.config.title.clone(),
            description: site.config.description.clone().unwrap_or("".to_string()),
            url: site.config.url.clone()
        }
    }
}

#[derive(Serialize, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Page<'a> {
    content: &'a String,
    title: &'a String,
    backlinks: &'a Vec<Link>,
    url: &'a String,
    summary: &'a Option<String>,
    date_created: String,
    date_updated: String,
    pub date_created_timestamp: i64,
    date_updated_timestamp: i64,
    tags: &'a Vec<String>,
    toc: &'a Vec<Link>,
    related_files: &'a Vec<Link>,
    is_section: bool
}

impl Page<'_> {
    pub fn new(md_file: &MdFile) -> Page {
        Page {
            content: &md_file.html,
            title: &md_file.frontmatter.title,
            backlinks: &md_file.backlinks,
            url: &md_file.full_url,
            summary: &md_file.frontmatter.summary,
            date_created: util::naive_date_to_str(md_file.frontmatter.date_created),
            date_updated: util::naive_date_to_str(md_file.frontmatter.date_updated),
            date_created_timestamp: md_file.frontmatter.date_created_timestamp,
            date_updated_timestamp: md_file.frontmatter.date_updated_timestamp,
            tags: &md_file.frontmatter.tags,
            toc: &md_file.toc,
            related_files: &md_file.related_files,
            is_section: md_file.is_section

        }
    }
}

/// A trimmed down version of MDFile, to be accessed in section files (_index.md)

#[derive(Serialize, Debug)]
pub struct SectionPage<'a> {
    pub pages: Vec<Page<'a>>,
}


impl SectionPage<'_> {
    pub fn new(pages: Vec<Page>) -> SectionPage {
        SectionPage { pages }
    }
}
