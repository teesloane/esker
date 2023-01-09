use std::path::Path;
use serde::Serialize;
use tera::Tera;
use crate::{util, link::Link, md_file::MdFile};

pub fn load_templates(dir_templates: &Path) -> Tera {
    let template_path = format!("{}/**/*.html", util::path_to_string(dir_templates));
    let mut tera = match Tera::new(&template_path) {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            util::exit();
        }
    };
    tera.autoescape_on(vec![]);
    if tera.templates.is_empty() {
        println!("\nError: No templates found in {:?}\n", template_path);
        util::exit();
    }
    tera
}


// get_template
// Returns the name of a template (to later render), provided it's found
// in the tera instance.
pub fn get_name(tera: &Tera, template: &str) -> String {
    let template_with_html = format!("{}.html", &template);
    let default_template_name = "default.html".to_string();
    if tera.get_template_names().any(|x| x == template_with_html) {
        return template_with_html;
    } else {
        return default_template_name
    }
}


// Structs for tera
//

#[derive(Serialize)]
pub struct Page<'a> {
    content: &'a String,
    title: &'a String,
    backlinks: &'a Vec<Link>,
    url: &'a String,
    summary: &'a Option<String>,
    date_created: String,
    date_updated: String,
    date_created_timestamp: i64,
    date_updated_timestamp: i64,
    tags: &'a Vec<String>
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
            tags: &md_file.frontmatter.tags
        }
    }
}

/// A trimmed down version of MDFile, to be accessed in section files (_index.md)

#[derive(Serialize)]
pub struct SectionPage<'a> {
  // front_matter: Frontmatter
  // file_url:  String,
  pages: Vec<Page<'a>>
}

impl SectionPage<'_> {
    pub fn new(pages: Vec<Page>) -> SectionPage {
        SectionPage {
            pages
        }
    }
}
