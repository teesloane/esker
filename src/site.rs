use colored::*;
use syntect::html;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::{env, fs::create_dir_all, path::PathBuf};



use crate::parser::syntax_highlight::THEMES;
// use crate::link::SiteLinks;
use crate::{config::Config, util};
use crate::{
    errors::Errors,
    frontmatter::Frontmatter,
    link::{Link, SiteLinks},
    md_file::MdFile,
    new_site,
    parser
};

use syntect::highlighting::ThemeSet;

#[derive(Debug)]
pub struct Site {
    /// The dir we are running the site in.
    pub dir: PathBuf,
    /// a list of markdown paths to process
    markdown_files_paths: Vec<PathBuf>,
    /// markdown as a struct of data and metadata
    pub markdown_files: HashMap<PathBuf, Vec<MdFile>>,
    /// files that have invalid frontmatter:
    invalid_files: Vec<PathBuf>,
    /// esker directory that gets generated in the vault: _esker
    pub dir_esker: PathBuf,
    /// esker directory for templates
    pub dir_esker_templates: PathBuf,
    /// out_path: _esker/_site
    pub dir_esker_build: PathBuf,
    /// _esker/public
    dir_esker_public: PathBuf,
    /// where public will go in _site.
    dir_esker_build_public: PathBuf,
    /// _esker/_site/<tag_url>/*tag_files.html
    pub dir_esker_build_tags: Option<PathBuf>,
    ///errorz
    pub errors: Errors,
    /// templating enginge
    pub tera: tera::Tera,
    /// links: internal and external
    pub links: SiteLinks,
    /// user config stuff
    pub config: Config,
    /// All tags, as collected from frontmatter (TODO: not from content yet!)
    pub tags: HashMap<String, Vec<Link>>,

    /// Sitemap of links to be injected into the Tera context.
    pub template_sitemap: Vec<Link>

}

impl Site {
    pub fn build(dir: Option<PathBuf>) -> Site {
        let cwd: PathBuf;
        if let Some(dir) = dir {
            cwd = dir;
        } else {
            cwd = env::current_dir().unwrap();
        }

        let esker_dir = cwd.clone().join("_esker");
        let dir_esker_build =  esker_dir.join("_site");
        let esker_dir_templates = esker_dir.join("templates");
        let user_config = Config::new(&cwd);

        let mut dir_esker_tags: Option<PathBuf>;
        if let Some(tags_dir) = &user_config.tags_url {
            let path_from_config = Path::new(&tags_dir).to_path_buf();
            let res = dir_esker_build.join(path_from_config);
            dir_esker_tags = Some(res);
        } else {
            dir_esker_tags = None;
        }

        let mut site = Site {
            dir: cwd.clone(),
            markdown_files_paths: Vec::new(),
            markdown_files: HashMap::new(),
            invalid_files: Vec::new(),
            dir_esker_templates: esker_dir_templates.clone(),
            dir_esker_build,
            dir_esker_public: esker_dir.join("public"),
            dir_esker_build_public: esker_dir.join("_site/public"),
            dir_esker: esker_dir,
            dir_esker_build_tags: dir_esker_tags,
            errors: Errors::new(),
            tera: crate::templates::load_templates(&esker_dir_templates),
            config: user_config,
            links: SiteLinks::new(),
            tags: HashMap::new(),
            template_sitemap: Vec::new()

        };

        site.create_required_directories_for_build();
        site.load_files();
        site.build_tag_pages();
        site.create_theme_css();
        site.cp_data();
        site.cp_public();

        return site;
    }

    fn create_required_directories_for_build(&self) {
        create_dir_all(self.dir_esker.clone()).unwrap();
        create_dir_all(self.dir_esker_public.clone()).unwrap();
        create_dir_all(self.dir_esker_build_public.clone()).unwrap();

        if let Some(attachment_dir) = &self.config.attachment_directory {
            let dir_attachments_build = self.dir_esker.join(attachment_dir);
            create_dir_all(dir_attachments_build).unwrap();
        }
    }

    /// For now we shell out to cp on unix because I don't want to figure this out in rust.
    /// copies data and public folder to their respective destinations
    pub fn cp_data(&mut self) {
        if let Some(attachment_dir) = &self.config.attachment_directory {
            let dir_attachments = self.dir.join(attachment_dir);
            Command::new("cp")
                .arg("-n")
                .arg("-r")
                .arg(dir_attachments.display().to_string())
                .arg(self.dir_esker_build.display().to_string())
                .output()
                .expect("Internal error: failed to copy data directory to _site.");
        }
    }

    /// build_tag_pages will render html pages for each tag,
    /// and is given a tera context with access to tagged_items;
    /// this allows users to generates html page per tag, that can
    /// link to each page that is thusly tagged.
    fn build_tag_pages(&self) {
        if let Some(dir_tags) = &self.dir_esker_build_tags {
            fs::create_dir_all(dir_tags).expect("failed to create tags directory");

            for (tag_name, vec_of_tagged_items) in &self.tags {
                let mut ctx = tera::Context::new();
                ctx.insert("baseurl", &self.config.url.clone());
                ctx.insert("tags", &self.tags);
                ctx.insert("tag", &tag_name);
                ctx.insert("sitemap", &self.template_sitemap);

                let tag_file_name = Path::new(tag_name).with_extension("html");
                let out_path = dir_tags.join(tag_file_name);
                let rendered_template = self.tera.render("tags.html", &ctx).unwrap();
                fs::write(out_path, rendered_template).unwrap();
            }
        }
    }

    pub fn cp_public(&mut self) {
        // for some reason I need to create _site/dest so cp works...
        create_dir_all(self.dir_esker_build_public.clone()).unwrap();
        Command::new("cp")
            .arg("-n")
            .arg("-r")
            .arg(self.dir_esker_public.display().to_string())
            .arg(self.dir_esker_build.display().to_string())
            .output()
            .expect("Internal error: failed to copy data directory to _site.");
    }

    // Fetches all the file paths with a glob
    // then iterates over them and loads them into the struct's memory.
    pub fn load_files(&mut self) {
        let markdown_files_paths = util::load_files(&self.dir, "**/*.md");

        let markdown_files_paths_filtered: Vec<_> = markdown_files_paths
            .iter()
            .filter(|f| self.is_in_private_folder(f))
            .collect();
        let mut markdown_files: HashMap<PathBuf, Vec<MdFile>> = HashMap::new();
        let mut invalid_files: Vec<PathBuf> = Vec::new();

        // collect all files and push them into the map.
        markdown_files_paths_filtered.iter().for_each(|f| {
            if let Some(fm) = Frontmatter::new(self, f) {
                let read_file = fs::read_to_string(f).expect("Unable to open file");
                let md_file = MdFile::new(self, read_file, f.to_path_buf(), fm);

                if md_file.frontmatter.publish {
                    if let Some(vec_of_files) = markdown_files.get_mut(&md_file.web_path_parents) {
                        self.collect_tags_from_frontmatter(&md_file);
                        self.template_sitemap.push(Link::new_tag_link_from_md_file(&md_file));
                        vec_of_files.push(md_file);


                    } else {
                        self.collect_tags_from_frontmatter(&md_file);
                        markdown_files.insert(md_file.web_path_parents.clone(), vec![md_file]);
                    }
                }
            } else {
                invalid_files.push(f.to_path_buf().clone());
            }
        });

        // Loop #1 - Let's get all the metadata, for things like backlinks, tags, etc.

        for (path, vec_md_files) in &mut markdown_files {
            for f in vec_md_files {
                if f.frontmatter.publish {
                    f.collect_metadata(self);
                }
            }
        }

        // TODO: not sure how to not have to clone this.
        let markdown_files_clone = markdown_files.clone();

        // Loop #2 - Let's render it!
        for (path, vec_md_files) in &mut markdown_files {
            for f in vec_md_files {
                if f.frontmatter.publish {
                    if f.is_section {
                    f.get_backlinks_for_file(self);
                        f.write_section_html(self, &markdown_files_clone);
                    } else {
                    f.get_backlinks_for_file(self);
                        f.write_html(self);
                    }
                }
            }
        }

        // on completion, we can now store the temporary data structures into self for future ref.
        self.markdown_files_paths = markdown_files_paths;
        self.markdown_files = markdown_files;
        self.invalid_files = invalid_files;

        if self.errors.has_errors() {
            self.errors.report_errors();
        }
    }

    fn collect_tags_from_frontmatter(&mut self, md_file: &MdFile) {
        for tag in &md_file.frontmatter.tags {
            let new_tag_link = Link::new_tag_link_from_md_file(&md_file);

            if let Some(list_of_links_for_tag) = self.tags.get_mut(tag) {
                list_of_links_for_tag.push(new_tag_link)

            } else  {
                self.tags.insert(tag.clone(), vec![new_tag_link]);

            }
        }
    }

    pub fn build_with_baseurl(&self, web_path: String) -> String {
        return format!("{}/{}", self.config.url, web_path);
    }

    // used to add links to the internal global links list.
    pub fn add_link(&mut self, link: Link) {
        if link.is_internal {
            self.links.internal.push(link)
        } else {
            self.links.external.push(link)
        }
    }

    /// filter out files that are in the private folder.
    pub fn is_in_private_folder(&self, file_source: &PathBuf) -> bool {
        if let Some(ignored_dirs) = &self.config.ignored_directories {
            // add the full path to all ignored _dirs
            let ignored_dirs: Vec<PathBuf> = ignored_dirs
                .iter()
                .map(|d| self.dir.join(Path::new(d)))
                .collect();

            let current_file_path = PathBuf::from(file_source);
            let mut ancestors = current_file_path.ancestors();
            !ancestors.any(|f| ignored_dirs.contains(&PathBuf::from(f)))
        } else {
            false
        }
    }

    fn create_theme_css(&self) {
        if let Some(theme) = THEMES.themes.get("zenburn") {
            let css = html::css_for_theme_with_class_style(theme, html::ClassStyle::SpacedPrefixed{prefix: "syntax-"}).unwrap();
            let css_output_path = Path::join(&self.dir_esker_public, Path::new("css/syntax-theme-dark.css"));
            fs::write(css_output_path, &css).expect("Unable to write css theme file");
        }

        if let Some(theme) = THEMES.themes.get("solarized-light") {
            let css = html::css_for_theme_with_class_style(theme, html::ClassStyle::SpacedPrefixed{prefix: "syntax-"}).unwrap();
            let css_output_path = Path::join(&self.dir_esker_public, Path::new("css/syntax-theme-light.css"));
            fs::write(css_output_path, &css).expect("Unable to write css theme file");
        }
    }

    // -- New Site generation

    // creates an _esker site and template files
    pub fn init(dir: Option<PathBuf>) {
        let cwd: PathBuf;
        if let Some(dir) = dir {
            cwd = dir;
        } else {
            cwd = env::current_dir().unwrap();
        }

        let dir_esker = cwd.join("_esker");
        if fs::metadata(&dir_esker).is_ok() {
            println!(
                "{}: An '_esker' site already exists in this directory.",
                " Failed ".yellow().on_black()
            );
        } else {
            let dirs = vec![
                "templates/",
                "templates/partials",
                "sass",
                "public/css",
                "public/js",
                "_site",
            ];

            let mut files = HashMap::new();
            files.insert(String::from("public/js/main.js"), new_site::DEFAULT_JS);
            files.insert(String::from("public/css/main.css"), new_site::DEFAULT_CSS);
            files.insert(String::from("templates/base.html"), new_site::BASE_HTML);
            files.insert(String::from("templates/default.html"), new_site::DEFAULT_HTML);
            files.insert(String::from("templates/tags.html"), new_site::TAGS_HTML);
            files.insert(String::from("templates/list.html"), new_site::LIST_HTML);
            files.insert(String::from("config.yaml"), new_site::CONFIG_YAML);

            // Map over the above strings, turn them into paths, and create them.
            for &dir in &dirs {
                let joined_dir = dir_esker.join(dir);
                fs::create_dir_all(joined_dir).expect("Couldn't create a new firn, directory");
            }

            for (filename, file_contents) in files {
                let joined_dir = dir_esker.join(filename);
                fs::write(joined_dir, file_contents)
                    .expect("Unable to write new site layout files.");
            }

            println!(
                "{}: created a new esker site at: {:?}",
                " Success ".green().on_black(),
                dir_esker
            );
        }
    }
}
