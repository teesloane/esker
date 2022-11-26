use std::{path::PathBuf, env};
use std::fs;

use crate::util;

#[derive(Debug)]
pub struct Site {
    pub dir: PathBuf,
    markdown_files_paths: Vec<PathBuf>,
    markdown_files_raw: Vec<String>
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
            markdown_files_raw: Vec::new(),
        };

        site.load_files();
        return site
    }

    // Fetches all the file paths with a glob
    // then iterates over them and loads them into the struct's memory.
    pub fn load_files(&mut self) {
        self.markdown_files_paths = util::load_files(&self.dir, "**/*.md");

        let markdown_files: Vec<_> = self.markdown_files_paths.iter().map(|f| {
            // let file_path = f.clone();
            let read_file = fs::read_to_string(f).expect("Unable to open file");
            return read_file;
        }).collect();

        self.markdown_files_raw = markdown_files;
    }
}
