use std::fs;
use std::{env, path::PathBuf};

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
    baseurl: String,
    /// out_path
    pub dir_build: PathBuf,
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

        let mut site = Site {
            dir: cwd.clone(),
            markdown_files_paths: Vec::new(),
            markdown_files: Vec::new(),
            invalid_files: Vec::new(),
            // TODO: make it possible to pass custom dir.
            // TODO: maybe build the output to be a parent up from dir_vault
            dir_build: cwd.join("_site"),
            dir_vault: cwd,

            baseurl: "foo.com".to_string(),
            errors: Errors::new(),
        };

        site.load_files();

        // println!("{:#?}", site);

        return site;
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

        self.markdown_files_paths = markdown_files_paths;
        self.markdown_files = markdown_files;
        self.invalid_files = invalid_files;

        // for each file, now that we have global data...render out their html
        for mut f in &mut self.markdown_files {
            f.write_html();
        }

        if self.errors.has_errors() {
            self.errors.report_errors();
        }
    }

    pub fn build_with_baseurl(&self, web_path: String) -> String {
        return format!("{}/{}", self.baseurl, web_path);
    }
}
