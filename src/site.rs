use std::fs;
use std::process::Command;
use std::{env, fs::create_dir_all, path::PathBuf};

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
    /// where static will go in _site.
    dir_esker_build_public: PathBuf,

    /// dir_vault - where all your markdown files are (your obsidian vault)
    dir_vault: PathBuf,
    ///errorz
    pub errors: Errors,
}

impl Site {
    pub fn new(dir: Option<PathBuf>) -> Site {
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
            // TODO: make it possible to pass custom dir.
            dir_vault: cwd.clone(),
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
    /// copies data and static folder to their respective destinations
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
}
