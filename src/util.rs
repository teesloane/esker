use chrono::prelude::{DateTime, Local};
use chrono::NaiveDateTime;
use glob::glob;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn load_files(cwd: &Path, pattern: &str) -> Vec<PathBuf> {
    let pattern_path = cwd.join(pattern);
    let pattern_path_str = pattern_path.to_str().unwrap();

    glob(pattern_path_str).unwrap().flatten().collect()
}

// steal code: https://stackoverflow.com/a/64148190
pub fn iso8601(st: std::time::SystemTime) -> String {
    let dt: DateTime<Local> = st.into();
    format!("{}", dt.format("%F"))
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn get_time_in_ms(s: SystemTime) -> std::time::Duration {
    s.duration_since(UNIX_EPOCH).expect("Time went backwards")
}

pub fn exit() -> ! {
    std::process::exit(1);
}

pub fn path_to_string(p: &Path) -> String {
    p.display().to_string()
}

pub fn naive_date_to_str(ndt: NaiveDateTime) -> String {
    return ndt.format("%Y-%m-%d %H:%M").to_string();
}

// when provided the current directory esker is run in (ex: /Users/my_site/test-site)
// turns p: "/Users/tees/development/tees/esker/test_site/posts/first_post.md",
// into: -> posts
// TODO: change strip_pwd -> strip_cwd
// TODO: the name of this should describe that it removes the pwd and the file...
// just basically gives you the web path.
pub fn strip_pwd(pwd: &Path, p: &Path) -> PathBuf {
    p.strip_prefix(pwd).unwrap().parent().unwrap().to_path_buf()
}

#[cfg(test)]
mod tests {
    use std::{path::{Path, PathBuf}, env};
    use chrono::NaiveDateTime;
    use crate::util::{self, load_files};

    use super::strip_pwd;

    #[test]
    fn test_strip_pwd() {
        let full_path = PathBuf::from("/Users/tees/development/tees/esker/test_site");
        let short_path =
            PathBuf::from("/Users/tees/development/tees/esker/test_site/posts/private file.md");
        assert_eq!(strip_pwd(&full_path, &short_path), PathBuf::from("posts"));
    }

    // NOTE: this is a silly test 🐱.
    #[test]
    fn test_naive_date_to_str() {
        let example_date_str = "2022-12-01 11:50";
        let ndt = NaiveDateTime::parse_from_str(example_date_str, "%Y-%m-%d %H:%M").unwrap();
        let res = util::naive_date_to_str(ndt);
        assert_eq!(res, example_date_str);
    }

    #[test]
    fn test_load_files() {
        let cwd = env::current_dir().unwrap();
        let res = load_files(&cwd, "tests/example_site/**/*.md");
        assert!(res.len() > 0);
    }
}
