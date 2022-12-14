use crate::util;
use chrono::prelude::{DateTime, Local, NaiveDateTime};
use std::fs;
use std::path::PathBuf;

use colored::*;

#[derive(Debug)]
pub struct Frontmatter {
    title: String,
    filepath: PathBuf,
    summary: Option<String>,
    tags: Vec<String>,
    published: bool,
    date_created: NaiveDateTime,
    date_updated: NaiveDateTime,
}

impl Frontmatter {
    pub fn new(md_file_path: &PathBuf) -> Option<Frontmatter> {
        let metadata = fs::metadata(md_file_path).unwrap();
        let mut has_valid_fm = true;

        let date_created = metadata.created().expect("failed to get created time.");
        let date_created: DateTime<Local> = date_created.clone().into();
        let date_created: NaiveDateTime = date_created.naive_local();

        let date_updated = metadata.modified().expect("failed to get modified time.");
        let date_updated: DateTime<Local> = date_updated.clone().into();
        let date_updated: NaiveDateTime = date_updated.naive_local();

        let mut fm = Frontmatter {
            title: std::ffi::OsString::into_string(
                md_file_path.file_stem().unwrap().to_os_string(),
            )
            .unwrap(),
            filepath: md_file_path.clone(),
            date_created,
            date_updated,
            summary: None,
            published: true,
            tags: Vec::new(),
        };
        let mut capturing = false;

        if let Ok(lines) = util::read_lines(md_file_path) {
            for line in lines {
                if let Ok(line) = line {
                    if line != "---" && capturing == false {
                        has_valid_fm = false;
                        break;
                    }

                    if line == "---" && capturing == false {
                        capturing = true;
                        continue;
                    }

                    fm.get_key_value_from_line(&line);

                    if line == "---" && capturing == true {
                        break;
                    }
                }
            }
        }

        if has_valid_fm {
            return Some(fm);
        } else {
            return None;
        }
    }

    pub fn get_key_value_from_line(&mut self, line: &str) {
        match line.split_once(":") {
            Some((key, val)) => {
                let lhs = key.trim();
                let rhs = val.trim();

                match lhs {
                    "title" => self.title = rhs.trim().to_string(),
                    "date_created" => {
                        match NaiveDateTime::parse_from_str(rhs, "%Y-%m-%d %H:%M") {
                            Ok(date_created) => self.date_created = date_created,
                            Err(_) => {
                                println!("\n⚠️  File {}\ndoes not have {} in {} format: defaulting to file creation metadata.\n",
                                         self.filepath.clone().into_os_string().into_string().unwrap().yellow().on_black(),
                                         "date_created".red(),
                                         "%Y-%m-%d %H:%M".green()
                                );
                            }
                        }
                    },

                    "date_updated" => {

                        match NaiveDateTime::parse_from_str(rhs, "%Y-%m-%d %H:%M") {
                            Ok(date_updated) => self.date_updated = date_updated,
                            Err(_) => {
                                println!("\n⚠️  File {}\ndoes not have {} in {} format: defaulting to file last modified metadata.\n",
                                         self.filepath.clone().into_os_string().into_string().unwrap().yellow().on_black(),
                                         "date_updated".red(),
                                         "%Y-%m-%d %H:%M".green()
                                );
                            }
                        }
                    },

                    "summary" => {
                        self.summary = Some(rhs.to_string());
                    }

                    "published" => {
                        self.published = if rhs == "false" { false } else { true };
                    }

                    "tag" | "tags" => {
                        let tags = rhs.split(",");
                        let vec: Vec<_> = tags
                            .collect::<Vec<&str>>()
                            .iter()
                            .map(|tag| tag.trim().to_string())
                            .collect();
                        self.tags = vec;
                    }
                    _ => (),
                }
            }
            None => {}
        }
    }

    pub fn date_modified_str(&self) -> String {
        return self.date_updated.format("%Y-%m-%d %H:%M").to_string();
    }

    pub fn date_created_str(&self) -> String {
        return self.date_updated.format("%Y-%m-%d %H:%M").to_string();
    }

}
