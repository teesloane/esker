use std::collections::HashMap;
use std::{fs, path::PathBuf};

use crate::frontmatter::Frontmatter;
use crate::link::Link;
use crate::parser;
use crate::site::Site;
use crate::templates;
use slugify::slugify;
use std::io;
use lazy_static::lazy_static;
use std::io::{BufRead, BufReader};
use tera::Context;
    use regex::{Regex, Captures};

lazy_static! {
    static ref WIKILINK: Regex = Regex::new(r"\[\[([^\]\[:]+)\]\]").unwrap();
    static ref WIKILINK_WITH_PIPE: Regex =
        Regex::new(r"\[\[([^\]\[:]+)\|([^\]\[:]+)\]\]").unwrap();
}

#[derive(Debug, Clone)]
pub struct MdFile {
    pub raw: String,
    pub html: String,
    path: PathBuf,
    pub web_path_parents: PathBuf,
    /// TODO: doc me!
    pub web_path: PathBuf,
    out_path: PathBuf,
    pub frontmatter: Frontmatter,
    pub file_name_without_extension: PathBuf,
    pub full_url: String,
    /// if file is a _index.md, we say it's a section, which
    /// is given a different tera context to render.
    pub is_section: bool,
    pub backlinks: Vec<Link>,
    pub toc: Vec<Link>,
    pub related_files: Vec<Link>,
}

impl MdFile {
    pub fn new(site: &Site, raw_str: String, path: PathBuf, fm: Frontmatter) -> MdFile {
        // Life's too short to write good code:

        // Below we set up the file's "out" path for writing the eventual html file to disk
        // AS WELL as the "web_path" which is the part that follow your domain name: <mydomain.com>/this-is-the-web-path.

        // turns; /Users/tees/development/tees/esker/test-site/foo/bar.md" -> test-site/foo/
        let web_path_parents = crate::util::strip_pwd(&site.dir, &path);

        let filename = path.file_stem().unwrap().to_str().unwrap().to_string();
        let mut out_file_path_slugified = slugify!(&filename);
        // takes slugified file name and adds html extension
        let web_path_stem = PathBuf::from(out_file_path_slugified).with_extension("html");
        let out_path = PathBuf::from(&site.dir_esker_site)
            .join(web_path_parents.join(PathBuf::from(&web_path_stem)));

        // now let's make the full url.
        let url_path = web_path_parents
            .join(web_path_stem.clone())
            .into_os_string()
            .into_string()
            .unwrap();
        let full_url = site.build_with_baseurl(url_path);
        // end bad code byeeee

        let mut md_file = MdFile {
            raw: raw_str,
            html: String::from(""),
            path,
            web_path_parents,
            web_path: web_path_stem,
            out_path,
            frontmatter: fm,
            file_name_without_extension: PathBuf::from(filename.clone()),
            full_url,
            is_section: filename == "_index",
            backlinks: Vec::new(),
            toc: Vec::new(),
            related_files: Vec::new(),
        };

        md_file
            .set_raw_contents()
            .expect("Failed to set raw contents for file");

        md_file
    }

    /// collect links, tags, etc so that they are available the next pass when we render.
    pub fn parse_markdown_to_html(&mut self, site: &mut Site) {
        self.preprocess(site);
        let parsed_str = parser::new(self, site);
        self.html = parsed_str;
    }

    /// checks if this site uses wikilinks and if so, run a regex
    /// that converts all wikilinks to markdown links in .raw
    fn preprocess(&mut self, site: &mut Site) {
        println!("preprocessing {:#?}", self.frontmatter.title);
        // TODO: replace this with a check in the config if user has wikilinks
        let has_wikilinks = true;
        if has_wikilinks {
            // TODO: write some regex that transforms wikilinks to markdown links.
            let processed = WIKILINK.replace_all(&self.raw, |caps: &Captures| {
                let link_name = &caps[1];
                let markdown_link_url = site.flat_sitemap.get(&PathBuf::from(link_name));
                if let Some(url) = markdown_link_url {
                    return format!("[{}]({}.md)", &caps[1], url);
                } else {
                    println!("Did not find link in site map for {:?}", link_name);
                    return format!("[{}]({}.md)", &caps[1], &caps[1]);
                }

            });
            self.raw = processed.to_string();
        }
    }

    fn get_related_files(&mut self, site: &Site) {
        let mut related_files: Vec<Link> = Vec::new();
        for tag in &self.frontmatter.tags {
            if let Some(tags) = site.tags.get(tag) {
                for tag_link in tags {
                    if tag_link.url != self.full_url && !related_files.contains(tag_link) {
                        related_files.push(tag_link.clone());
                    }
                }
            }
        }
        self.related_files = related_files
    }

    /// enables creating "post list" type pages where the "section" context
    /// corresponds to every file in the dir. Only runs for files named _index.md in a dir.
    pub fn write_section_html(
        &mut self,
        site: &Site,
        markdown_files: &HashMap<PathBuf, Vec<MdFile>>,
    ) {
        if let Some(section_content) = markdown_files.get(&self.web_path_parents) {
            let serialized_pages: Vec<_> = section_content
                .iter()
                .filter(|md_file| !md_file.is_section)
                .map(templates::Page::new)
                .collect();

            self.get_related_files(site);
            let mut ctx = Context::new();
            ctx.insert("page", &templates::Page::new(self));
            ctx.insert("pages", &serialized_pages);
            ctx.insert("baseurl", &site.config.url.clone());
            ctx.insert("section", &templates::SectionPage::new(serialized_pages));
            ctx.insert("config", &templates::Config::new(site));
            ctx.insert("tags", &site.tags);
            ctx.insert("tags", &site.tags);
            ctx.insert("sitemap", &site.template_sitemap);

            let template_name = templates::get_name(&site.tera, &self.frontmatter.template);
            let rendered_template = site.tera.render(&template_name, &ctx).unwrap();
            let prefix = &self.out_path.parent().unwrap();
            fs::create_dir_all(prefix).unwrap();
            fs::write(&self.out_path, rendered_template).expect("Unable to write file");
        }
    }

    /// writes a file to it's specified output path.
    pub fn write_html(&mut self, site: &mut Site) {
        self.get_related_files(site);

        let mut ctx = Context::new();
        ctx.insert("page", &templates::Page::new(self));
        ctx.insert("baseurl", &site.config.url.clone());
        ctx.insert("tags", &site.tags);
        ctx.insert("config", &templates::Config::new(site));
        ctx.insert("sitemap", &site.template_sitemap);
        let template_name = templates::get_name(&site.tera, &self.frontmatter.template);
        let rendered_template = site.tera.render(&template_name, &ctx).unwrap();

        let prefix = &self.out_path.parent().unwrap();
        fs::create_dir_all(prefix).unwrap();
        fs::write(&self.out_path, rendered_template).expect("Unable to write file");
    }

    pub fn get_backlinks_for_file(&mut self, site: &Site) {
        let mut out: Vec<Link> = Vec::new();
        for g_link in &site.links.internal {
            if let Some(originating_file_url) = &g_link.originating_file_url {
                if g_link.url == self.full_url
                    && self.full_url != originating_file_url.clone()
                    && !out.contains(g_link)
                {
                    out.push(g_link.clone());
                }
            }
        }
        self.backlinks = out
    }

    /// sets the "raw" contents field for the md_file to be the file without the frontmatter.
    fn set_raw_contents(&mut self) -> io::Result<()> {
        let input_file = fs::File::open(self.path.clone())?;
        let mut output: Vec<_> = Vec::new();
        let mut reader = BufReader::new(input_file);
        let mut in_frontmatter = false;

        for (_i, line) in reader.lines().enumerate() {
            // Write the line to the output file
            let line = line?;
            if line == "---" && !in_frontmatter {
                in_frontmatter = true
            } else if line == "---" && in_frontmatter {
                in_frontmatter = false;
                continue;
            }

            if !in_frontmatter {
                output.push(line)
            }
        }

        self.raw = output.join("\n");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use lazy_static::lazy_static;
    use regex::{Regex, Captures};

    lazy_static! {
        static ref WIKILINK: Regex = Regex::new(r"\[\[([^\]\[:]+)\]\]").unwrap();
        static ref WIKILINK_WITH_PIPE: Regex =
            Regex::new(r"\[\[([^\]\[:]+)\|([^\]\[:]+)\]\]").unwrap();
    }

    #[test]
    fn test_wikilink() {
        assert!(WIKILINK.is_match("[[I am a link]]"));
        let ex = "I like [[foo/Oranges]] and here's [[another link]]";

        let result = WIKILINK.replace_all(ex, |caps: &Captures| {
            print!("\n{:?}", &caps);
            format!("[{}]({})", &caps[1], &caps[1])
        });

        println!("\n\n caps are {:#?}", result);
    }
}
