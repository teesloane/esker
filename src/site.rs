use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::{env, fs::create_dir_all, path::PathBuf};
use colored::*;

use crate::util;
use crate::{errors::Errors, frontmatter::Frontmatter, md_file::MdFile};

#[derive(Debug)]
pub struct Site {
    /// The dir we are running the site in. // TODO: merge this with dir_vault?
    pub dir: PathBuf,
    /// a list of markdown paths to process
    markdown_files_paths: Vec<PathBuf>,
    /// markdown as a struct of data and metadata
    markdown_files: Vec<MdFile>,
    /// files that have invalid frontmatter:=
    invalid_files: Vec<PathBuf>,
    /// baseurl
    pub baseurl: String,
    /// esker directory that gets generated in the vault: _esker
    pub dir_esker: PathBuf,
    /// out_path: _esker/_site
    pub dir_esker_build: PathBuf,
    /// where users stores their attachments: <my_vault>/attachments
    pub dir_attachments: PathBuf,
    /// where attachments will go in _site.
    pub dir_esker_build_attachments: PathBuf,
    /// _esker/public
    dir_esker_public: PathBuf,
    /// where public will go in _site.
    dir_esker_build_public: PathBuf,
    ///errorz
    pub errors: Errors,
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

        let mut site = Site {
            dir: cwd.clone(),
            markdown_files_paths: Vec::new(),
            markdown_files: Vec::new(),
            invalid_files: Vec::new(),
            dir_attachments: cwd.join("attachments"),
            dir_esker_build: esker_dir.join("_site"),
            dir_esker_build_attachments: esker_dir.join("_site/attachments"),
            dir_esker_public: esker_dir.join("public"),
            dir_esker_build_public: esker_dir.join("_site/public"),
            dir_esker: esker_dir,
            baseurl: "http://127.0.0.1:8080".to_string(),
            errors: Errors::new(),
        };

        site.create_required_directories_for_build();
        site.load_files();
        site.cp_data();
        return site;
    }

    fn create_required_directories_for_build(&self) {
        create_dir_all(self.dir_esker.clone()).unwrap();
        create_dir_all(self.dir_esker_public.clone()).unwrap();
        create_dir_all(self.dir_esker_build_attachments.clone()).unwrap();
        create_dir_all(self.dir_esker_build_public.clone()).unwrap();
    }

    /// For now we shell out to cp on unix because I don't want to figure this out in rust
    /// and windows support for Firn doesn't exist in the clojure version anyway.
    /// copies data and public folder to their respective destinations
    pub fn cp_data(&mut self) {
        Command::new("cp")
            .arg("-n")
            .arg("-r")
            .arg(self.dir_attachments.display().to_string())
            .arg(self.dir_esker_build.display().to_string())
            .output()
            .expect("Internal error: failed to copy data directory to _site.");
    }


    pub fn cp_static(&mut self) {
        // for some reason I need to create _site/dest so cp works...
        create_dir_all(self.dir_esker_build_public.clone()).unwrap();
        Command::new("cp")
            .arg("-n")
            .arg("-r")
            .arg(self.dir_esker_public.display().to_string())
            .arg(self.dir_esker_build_public.display().to_string())
            .output()
            .expect("Internal error: failed to copy data directory to _site.");
    }

    // Fetches all the file paths with a glob
    // then iterates over them and loads them into the struct's memory.
    pub fn load_files(&mut self) {
        // self.markdown_files_paths = util::load_files(&self.dir, "**/*.md");
        let markdown_files_paths = util::load_files(&self.dir, "**/*.md");
        let mut markdown_files: Vec<MdFile> = Vec::new();
        let mut invalid_files: Vec<PathBuf> = Vec::new();

        // collect all files and their metadata.
        markdown_files_paths.iter().for_each(|f| {
            if let Some(fm) = Frontmatter::new(self, f) {
                let read_file = fs::read_to_string(f).expect("Unable to open file");
                let md_file = MdFile::new(self, read_file, f.to_path_buf(), fm);
                markdown_files.push(md_file);
            } else {
                invalid_files.push(f.clone());
            }
        });

        // for each file, now that we have global data...render out their html
        for mut f in &mut markdown_files {
            f.write_html(self);
        }

        self.markdown_files_paths = markdown_files_paths;
        self.markdown_files = markdown_files;
        self.invalid_files = invalid_files;

        if self.errors.has_errors() {
            self.errors.report_errors();
        }
    }

    pub fn build_with_baseurl(&self, web_path: String) -> String {
        return format!("{}/{}", self.baseurl, web_path);
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
        println!("{}: An '_esker' site already exists in this directory.", " Failed ".yellow().on_black());
    } else {
        let dirs = vec!["layouts/",
                        "layouts/partials",
                        "sass",
                        "public/css",
                        "public/js",
                        "_site"];

        let mut files = HashMap::new();
        files.insert(String::from("layouts/partials/head.html"), PARTIAL_HEAD);
        files.insert(String::from("public/js/main.js"), DEFAULT_JS);
        files.insert(String::from("public/css/main.css"), DEFAULT_CSS);
        files.insert(String::from("layouts/default.html"), DEFAULT_HTML);
        files.insert(String::from("config.yaml"), CONFIG_YAML);

        // Map over the above strings, turn them into paths, and create them.
        for &dir in &dirs {
            let joined_dir = dir_esker.join(dir);
            fs::create_dir_all(joined_dir).expect("Couldn't create a new firn, directory");
        }

        for (filename, file_contents) in files {
            let joined_dir = dir_esker.join(filename);
            fs::write(joined_dir, file_contents).expect("Unable to write new site layout files.");
        }

        println!("{}: created a new esker site at: {:?}", " Success ".green().on_black(),  dir_esker);
    }

    }
}

const CONFIG_YAML: &str = r#"# site-wide configuration:
site:
  # your site's url
  url: "http://localhost:8080"
  # site title
  title: "My Site"
  # your site description.
  description: "My Site Description"
  # name of the directory where you store attachments
  attachment_directory: "attachments"
"#;


const PARTIAL_HEAD: &str = r#"<html>
  <head>
    <meta charset="utf-8">
    <title>{{ title }} - My Site</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="apple-touch-icon" href="/apple-touch-icon.png">
    <script src="{{config.site.url}}/public/js/main.js"></script>
    <link rel="stylesheet" href="{{config.site.url}}/public/css/main.css" type="text/css" media="screen" />
    <style>
    </style>
  </head>
"#;


const DEFAULT_HTML: &str = r#"{% import "macros.html" as macros %}
<html>
  {% include "partials/head.html" %}
  <body style="display: flex;">
    <main>
      {{render()}}
    </main>
  </body>
</html>
"#;

const DEFAULT_JS: &str = r#"
"#;

const DEFAULT_CSS: &str = r#"body{
  color: #333;
  background: #efefef;
  max-width: 800px;
  margin: 0 auto;
}

section {
  padding-bottom: 32px;
}

ul, ol {
  padding-left: 16px;
}

img { max-width: 600px; }
"#;
