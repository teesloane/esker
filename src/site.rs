use colored::*;
use hotwatch::Event;
use slugify::slugify;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::{env, fs::create_dir_all, path::PathBuf};
use syntect::html;

use glob::glob;

use crate::parser::syntax_highlight::THEMES;

use crate::templates::{self, Page};
use crate::{config::Config, util};
use crate::{
    errors::Errors,
    frontmatter::Frontmatter,
    link::{Link, SiteLinks},
    md_file::MdFile,
    new_site,
};
use crate::{Cli, Commands};

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
    pub dir_esker_site: PathBuf,
    /// <cwd>/<my_attachment_directory>. Might not exist (if user doesn't put it in config.)
    pub dir_attachments: Option<PathBuf>,
    /// _esker/public
    dir_esker_public: PathBuf,
    /// _esker/<my_attachment_folder_name>
    dir_esker_site_attachments: Option<PathBuf>,
    /// where public will go in _site.
    dir_esker_site_public: PathBuf,
    /// _esker/_site/<tag_url>/*tag_files.html
    pub dir_esker_site_tags: Option<PathBuf>,
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
    pub template_sitemap: Vec<Link>,
    pub attachments: Vec<Link>,
    /// Which command was run (build, watch, etc.)
    pub cli_command: Commands,
    /// the clap cli struct.
    pub cli: Cli,

    // TODO: document this later or change the name plz.
    // This is used just for the ability to convert wikilinks into markdown links.
    pub flat_sitemap: HashMap<PathBuf, Vec<String>>,
}

impl Site {
    pub fn new(cmd: crate::Commands, cli: crate::Cli) -> Self {
        let cwd: PathBuf;
        if let Some(dir) = cli.dir.clone() {
            cwd = dir;
        } else {
            cwd = env::current_dir().unwrap();
        }

        let esker_dir = cwd.join("_esker");
        let dir_esker_build = esker_dir.join("_site");
        let user_config = Config::new(&cwd, &cmd);

        let mut dir_esker_tags: Option<PathBuf>;
        if let Some(tags_dir) = &user_config.tags_url {
            let path_from_config = Path::new(&tags_dir).to_path_buf();
            let res = dir_esker_build.join(path_from_config);
            dir_esker_tags = Some(res);
        } else {
            dir_esker_tags = None;
        }

        let mut dir_attachments = None;
        let mut dir_esker_build_attachments = None;
        if let Some(attachment_dir) = &user_config.attachment_directory {
            dir_attachments = Some(cwd.join(attachment_dir));
            dir_esker_build_attachments = Some(dir_esker_build.join(attachment_dir));
        }

        let (dir_esker_templates, dir_esker_public) =
            Site::get_possible_theme_paths(&user_config, &esker_dir);

        Site {
            dir: cwd,
            markdown_files_paths: Vec::new(),
            markdown_files: HashMap::new(),
            invalid_files: Vec::new(),
            dir_esker_templates: dir_esker_templates.clone(),
            dir_esker_site: dir_esker_build,
            dir_attachments,
            dir_esker_public,
            dir_esker_site_attachments: dir_esker_build_attachments,
            dir_esker_site_public: esker_dir.join("_site/public"),
            dir_esker: esker_dir,
            dir_esker_site_tags: dir_esker_tags,
            errors: Errors::new(),
            tera: crate::templates::load_templates(&dir_esker_templates),
            config: user_config,
            links: SiteLinks::new(),
            tags: HashMap::new(),
            template_sitemap: Vec::new(),
            attachments: Vec::new(),
            cli,
            cli_command: cmd,
            flat_sitemap: HashMap::new(),
        }
    }

    fn get_possible_theme_paths(cfg: &Config, dir_esker: &Path) -> (PathBuf, PathBuf) {
        let templates = dir_esker.join("templates");
        let public = dir_esker.join("public");

        if let Some(theme) = &cfg.theme {
            let theme_folder = dir_esker.join(Path::new("themes")).join(theme);
            let theme_public = theme_folder.join("public");
            let theme_templates = theme_folder.join("templates");

            if !theme_folder.is_dir() {
                println!(
                    "The theme: '{}' does not exist in your _esker folder",
                    theme
                );
                util::exit()
            }

            if theme_public.is_dir() && theme_templates.is_dir() {
                return (theme_templates, theme_public);
            } else {
                println!(
                    "Please ensure that theme '{}' has a 'public' and 'templates' directory",
                    theme
                );
                util::exit()
            }
            // is the theme valid? -> check if there /templates and /public
        } else {
            return (templates, public);
        }
    }

    pub fn build(&mut self) {
        self.create_required_directories_for_build();
        self.load_files();
        self.build_tag_pages();
        self.create_theme_css();
        self.cp_data();
        self.cp_public();
        self.build_syndication_pages();

        if self.errors.has_errors() {
            self.errors.report_errors(self.cli.verbose);
        }
    }

    fn clear_site_for_rebuild(&mut self) {
        self.errors.clear();
        self.markdown_files.clear();
        self.markdown_files_paths.clear();
        self.invalid_files.clear();
        self.tags.clear();
        self.template_sitemap.clear();
    }

    fn rebuild(&mut self) {
        // reload config
        self.config = Config::new(&self.dir, &self.cli_command);
        let (dir_esker_templates, dir_esker_public) =
            Site::get_possible_theme_paths(&self.config, &self.dir_esker);
        self.dir_esker_templates = dir_esker_templates;
        self.dir_esker_public = dir_esker_public;

        fs::remove_dir_all(&self.dir_esker_site).expect("failed to delete _site");

        // rebuild
        self.clear_site_for_rebuild();
        self.tera = crate::templates::load_templates(&self.dir_esker_templates);
        self.build();
    }

    fn rebuild_markdown(&mut self) {
        self.clear_site_for_rebuild();
        self.load_files();
        self.build_tag_pages();
        self.build_syndication_pages();
    }

    fn create_required_directories_for_build(&self) {
        create_dir_all(self.dir_esker.clone()).unwrap();
        create_dir_all(self.dir_esker_public.clone()).unwrap();
        create_dir_all(self.dir_esker_site_public.clone()).unwrap();

        if let Some(attachment_dir) = &self.config.attachment_directory {
            let dir_attachments_build = self.dir_esker.join(attachment_dir);
            create_dir_all(dir_attachments_build).unwrap();
        }
    }

    pub fn cp_public(&mut self) {
        fs::remove_dir_all(&self.dir_esker_site_public).unwrap();
        create_dir_all(self.dir_esker_site_public.clone()).unwrap();
        Command::new("cp")
            .arg("-n")
            .arg("-r")
            .arg(self.dir_esker_public.display().to_string())
            .arg(self.dir_esker_site.display().to_string())
            .output()
            .expect("Internal error: failed to copy data directory to _site.");
    }

    /// For now we shell out to cp on unix because I don't want to figure this out in rust.
    /// copies data and public folder to their respective destinations
    pub fn cp_data(&mut self) {
        if let Some(dir_attachment_site) = &self.dir_esker_site_attachments {
            if Path::new(dir_attachment_site).is_dir() {
                fs::remove_dir_all(dir_attachment_site).unwrap();
            }
        }

        if let Some(attachment_dir) = &self.dir_attachments {
            Command::new("cp")
                .arg("-n")
                .arg("-r")
                .arg(attachment_dir.display().to_string())
                .arg(self.dir_esker_site.display().to_string())
                .output()
                .expect("Internal error: failed to copy data directory to _site.");
        }
        self.cleanup_unusued_attachments();
    }

    /// build_tag_pages will render html pages for each tag,
    /// and is given a tera context with access to tagged_items;
    /// this allows users to generate an html page per tag, that can
    /// link to each page that is thusly tagged.
    fn build_tag_pages(&self) {
        if let Some(dir_tags) = &self.dir_esker_site_tags {
            fs::create_dir_all(dir_tags).expect("failed to create tags directory");

            for tag_name in self.tags.keys() {
                if !tag_name.is_empty() {
                    let mut ctx = tera::Context::new();
                    ctx.insert("baseurl", &self.config.url.clone());
                    ctx.insert("tags", &self.tags);
                    ctx.insert("config", &templates::Config::new(self));
                    ctx.insert("tag", &tag_name);
                    ctx.insert("sitemap", &self.template_sitemap);

                    let tag_file_name = Path::new(tag_name).with_extension("html");
                    let out_path = dir_tags.join(tag_file_name);
                    let rendered_template = self.tera.render("tags.html", &ctx).unwrap();
                    fs::write(out_path, rendered_template).unwrap();
                }
            }
        }
    }

    /// responsible for rendering a feed.rss template using tera.
    fn build_syndication_pages(&mut self) {
        let mut all_pages: Vec<Page> = Vec::new();

        for md_files in self.markdown_files.values() {
            for md_file in md_files {
                let page = Page::new(md_file);
                all_pages.push(page);
            }
        }

        all_pages.sort_by(|a, b| b.date_created_timestamp.cmp(&a.date_created_timestamp));

        let mut ctx = tera::Context::new();

        ctx.insert("config", &templates::Config::new(self));
        ctx.insert("pages", &all_pages);

        let rendered_template = self.tera.render("feed.rss", &ctx).unwrap();
        let out_path = self.dir_esker_site.join("feed.rss");
        fs::write(out_path, rendered_template).unwrap();
    }

    // Fetches all the file paths with a glob
    // then iterates over them and loads them into the struct's memory.
    // TODO: break this into multiple functions
    pub fn load_files(&mut self) {
        let markdown_files_paths = util::load_files(&self.dir, "**/*.md");

        let markdown_files_paths_filtered: Vec<_> = markdown_files_paths
            .iter()
            .filter(|f| self.is_in_private_folder(f))
            .collect();
        let mut markdown_files: HashMap<PathBuf, Vec<MdFile>> = HashMap::new();
        let mut flat_sitemap: HashMap<PathBuf, Vec<String>> = HashMap::new();
        let mut invalid_files: Vec<PathBuf> = Vec::new();

        // Loop over files, collect metadata before processing.
        markdown_files_paths_filtered.iter().for_each(|f| {
            if let Some(fm) = Frontmatter::new(self, f) {
                let read_file = fs::read_to_string(f).expect("Unable to open file");
                let mut md_file = MdFile::new(self, read_file, f.to_path_buf(), fm);

                // pushes files into the markdown_files map
                if md_file.frontmatter.publish {
                    if let Some(vec_of_files) = markdown_files.get_mut(&md_file.web_path_parents) {
                        self.collect_tags_from_frontmatter(&md_file);
                        // TODO: why isn't this in the else block as well?
                        self.template_sitemap.push(Link::new_sitemap_link(&md_file));

                        Self::collect_flat_sitemap(&md_file, &mut flat_sitemap);
                        vec_of_files.push(md_file);
                    } else {
                        Self::collect_flat_sitemap(&md_file, &mut flat_sitemap);
                        self.collect_tags_from_frontmatter(&md_file);

                        markdown_files.insert(md_file.web_path_parents.clone(), vec![md_file]);
                    }
                }
            } else {
                invalid_files.push(f.to_path_buf());
            }
        });

        self.flat_sitemap = flat_sitemap;
        println!("{:#?}", self.flat_sitemap);

        // We parse outside of the above loop so that we can ensure that we have access to the flat sitemap
        for (path, vec_of_files) in &mut markdown_files {
            for mut md_file in vec_of_files {
                md_file.parse_markdown_to_html(self);
            }
        }

        // TODO (i tried, i don't know): not sure how to not have to clone this.
        let markdown_files_clone = markdown_files.clone();

        // Loop #2 - Let's render it!
        for vec_md_files in markdown_files.values_mut() {
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
    }

    fn collect_tags_from_frontmatter(&mut self, md_file: &MdFile) {
        for tag in &md_file.frontmatter.tags {
            let new_tag_link = Link::new_tag_link_from_md_file(md_file);

            if let Some(list_of_links_for_tag) = self.tags.get_mut(tag) {
                list_of_links_for_tag.push(new_tag_link)
            } else {
                self.tags.insert(tag.clone(), vec![new_tag_link]);
            }
        }
    }

    pub fn build_with_baseurl(&self, web_path: String) -> String {
        format!("{}/{}", self.config.url, web_path)
    }

    /// Allow us to pass k (which is the text from a wikilink) to find the linked item in the flat_sitemap
    /// it should return the equivalent markdown link to that file.
    /// Here be petit dragons, because of wikilinks being hard.
    pub fn get_item_from_flat_sitemap(&self, k: PathBuf) -> Option<String> {
        //WRITE BAD CODE ALL DAY EVERY DAY!
        println!("looking in flast sitemap for K: {:?}", k);
        if let Some(possible_md_paths) = self.flat_sitemap.get(&k) {
            // println!("posbmdf {:#?}", possible_md_paths);
            for f in possible_md_paths {
                if PathBuf::from(f) == k {
                    return Some(f.to_string());
                } else {
                    return Some(f.to_string());
                }
            }

            // if possible_md_paths.len() > 0 {
            //     return Some(possible_md_paths[0].clone());
            // }
        }
        return None;
    }

    // used to add links to the internal global links list.
    pub fn add_link(&mut self, link: Link) {
        if link.is_internal {
            self.links.internal.push(link)
        } else {
            self.links.external.push(link)
        }
    }

    pub fn add_attachment(&mut self, link: Link) {
        self.attachments.push(link);
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

    /// Runs after copying over dir_attachments, and checks to see
    /// if all the files in dir_attachments are also in self.attachments;
    /// if not, we delete them.
    fn cleanup_unusued_attachments(&self) {
        if let Some(site_attachments_dir) = &self.dir_esker_site_attachments {
            let glob_pattern = format!("{}/**/*", site_attachments_dir.display());
            let approved_attachments: Vec<String> = self
                .attachments
                .iter()
                .filter_map(|link| link.original.clone())
                .collect();
            for file in glob(&glob_pattern).unwrap() {
                match file {
                    Ok(pathbuf) => {
                        let trimmed_attachment_path = pathbuf
                            .strip_prefix(&self.dir_esker_site)
                            .unwrap()
                            .to_path_buf();
                        let attachment_str = &util::path_to_string(&trimmed_attachment_path);
                        let attachment_str_encoded: String =
                            url_escape::encode_fragment(attachment_str).into();

                        if !approved_attachments.contains(&attachment_str_encoded) {
                            if pathbuf.is_file() {
                                fs::remove_file(pathbuf).unwrap();
                            }
                        }
                    }
                    Err(_e) => (),
                }
            }
        }
    }

    fn create_theme_css(&self) {
        if let Some(theme) = THEMES.themes.get("zenburn") {
            let css = html::css_for_theme_with_class_style(
                theme,
                html::ClassStyle::SpacedPrefixed { prefix: "syntax-" },
            )
            .unwrap();
            let css_output_path = Path::join(
                &self.dir_esker_public,
                Path::new("css/syntax-theme-dark.css"),
            );
            fs::write(css_output_path, &css).expect("Unable to write css theme file");
        }

        if let Some(theme) = THEMES.themes.get("solarized-light") {
            let css = html::css_for_theme_with_class_style(
                theme,
                html::ClassStyle::SpacedPrefixed { prefix: "syntax-" },
            )
            .unwrap();
            let css_output_path = Path::join(
                &self.dir_esker_public,
                Path::new("css/syntax-theme-light.css"),
            );
            fs::write(css_output_path, &css).expect("Unable to write css theme file");
        }
    }

    pub fn handle_watch_event(&mut self, event: Event) {
        if let Event::Write(path) | Event::Create(path) | Event::Remove(path) = event {
            // NOTE: this removes the last element if it's a file and removes
            // all prefixing path parents from the current working directory.
            let stripped_path = util::strip_pwd(&self.dir, &path);

            if let Some(ext) = path.extension() {
                if ext == "md" {
                    self.rebuild_markdown();
                }
            }

            // handle public folder
            if path.starts_with(&self.dir_esker_public) {
                self.cp_public()
            }

            // handle templates and config file.
            if path.starts_with(&self.dir_esker_templates) {
                self.rebuild()
            }

            if let Some(filename) = path.file_name() {
                if filename == "config.yaml" {
                    self.rebuild()
                }
            }

            if let Some(dir_attachments) = &self.dir_attachments {
                let mut dir_attachments = dir_attachments.clone();
                if let Some(dir_attachments_name) = dir_attachments.file_name() {
                    if stripped_path.starts_with(dir_attachments_name) {
                        self.cp_data();
                    }
                }
            }
        }
    }

    // YEAH!
    //
    //   ,    /),
    //   (( -.((_))  _,)
    //   ,\`.'_  _`-','
    //   `.> <> <>  (,-
    //  ,',    |     `._,)
    // ((  )   |,   (`--'
    //  `'( ) _--_,-.\ SSt
    //     /,' \( )  `'
    //    ((    `\
    //     `
    //
    fn collect_flat_sitemap(md_file: &MdFile, flat_sitemap: &mut HashMap<PathBuf, Vec<String>>) {
        let md_rel_path_link = &md_file
            .web_path_parents
            .clone()
            .join(md_file.file_name_without_extension.clone())
            .to_path_buf();
        let md_rel_path_link_str = util::path_to_string(&md_rel_path_link);

        // println!("md_rel_path_link_str: {:?}", md_rel_path_link_str);

        //insert the full path first: /my/foo/celery
        if let Some(res) = flat_sitemap.get_mut(md_rel_path_link) {
            if !res.contains(&md_rel_path_link_str) {
                res.push(md_rel_path_link_str.clone())
            }
        } else {
            flat_sitemap.insert(md_rel_path_link.clone(), vec![md_rel_path_link_str.clone()]);
        }

        // try and insert just the file stem
        let file_stem = md_file.file_name_without_extension.clone();
        if let Some(res) = flat_sitemap.get_mut(&file_stem) {
            let x = util::path_to_string(&file_stem);

            if !res.contains(&md_rel_path_link_str)  {
                res.push(md_rel_path_link_str.clone());
                if  !res.contains(&x) {
                    res.push(x);
                }
            }
        } else {
            flat_sitemap.insert(file_stem, vec![md_rel_path_link_str.clone()]);
        }
    }
}
