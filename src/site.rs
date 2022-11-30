use std::{path::PathBuf, env};
use std::fs;

use crate::util;
use crate::md_file::{MdFile, Frontmatter};

#[derive(Debug)]
pub struct Site {
    pub dir: PathBuf,
    /// a list of markdown paths to process
    markdown_files_paths: Vec<PathBuf>,
    /// markdown as a struct of data and metadata
    markdown_files: Vec<MdFile>,
    /// files that have invalid frontmatter:=
    invalid_files: Vec<PathBuf>
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
            dir: cwd,
            markdown_files_paths: Vec::new(),
            markdown_files: Vec::new(),
            invalid_files: Vec::new(),
        };

        site.load_files();
        return site
    }

    // Fetches all the file paths with a glob
    // then iterates over them and loads them into the struct's memory.
    pub fn load_files(&mut self) {
        self.markdown_files_paths = util::load_files(&self.dir, "**/*.md");
        let mut markdown_files: Vec<MdFile> = Vec::new();
        let mut invalid_files: Vec<PathBuf> = Vec::new();

        self.markdown_files_paths.iter().for_each(|f| {
            let fm = Frontmatter::new(f);
            if let Some(fm) = Frontmatter::new(f) {
                let read_file = fs::read_to_string(f).expect("Unable to open file");
                let md_file  = MdFile::new(read_file, f.to_path_buf(), fm);
                markdown_files.push(md_file);
            } else {
                invalid_files.push(f.clone());
            }
        });

        self.markdown_files = markdown_files;
        self.invalid_files = invalid_files;
    }
}
