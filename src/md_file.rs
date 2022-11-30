use crate::util;
use chrono::prelude::{DateTime, Local, NaiveDateTime};
use std::fs;
use std::path::PathBuf;

use std::io;
use std::io::{BufRead, BufReader};

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
                        let date_created = NaiveDateTime::parse_from_str(rhs, "%Y-%m-%d %H:%M")
                            .expect("failed to parse datestring.");
                        self.date_created = date_created;
                    }

                    "date_updated" => {
                        let date_updated = NaiveDateTime::parse_from_str(rhs, "%Y-%m-%d %H:%M")
                            .expect("failed to parse datestring.");
                        self.date_updated = date_updated;
                    }
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

#[derive(Debug)]
pub struct MdFile {
    raw: String,
    path: PathBuf,
    frontmatter: Frontmatter,
}

impl MdFile {
    pub fn new(raw_str: String, path: PathBuf, fm: Frontmatter) -> MdFile {
        let mut md_file = MdFile {
            raw: raw_str,
            path,
            frontmatter: fm,
        };

        md_file.set_raw_contents().expect("Failed to set raw contents for file");

        return md_file;
    }

    /// sets the "raw" contents field for the md_file to be the file without the frontmatter.
    fn set_raw_contents(&mut self) -> io::Result<()> {
        let input_file = fs::File::open(self.path.clone())?;
        let mut output: Vec<_> = Vec::new();
        let mut reader = BufReader::new(input_file);
        let mut in_frontmatter = false;

        for (i, line) in reader.lines().enumerate() {
            // Write the line to the output file
            let line = line?;
            if line == "---" && in_frontmatter == false {
                in_frontmatter = true
            } else if line == "---" && in_frontmatter == true {
                in_frontmatter = false;
                continue;
            }

            if in_frontmatter == false {
                output.push(line)
            }
        }

        self.raw = output.join("\n");
        Ok(())
    }
}
