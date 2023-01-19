use chrono::prelude::{DateTime, Local};
use chrono::NaiveDateTime;
use glob::glob;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{self, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn load_files(cwd: &PathBuf, pattern: &str) -> Vec<PathBuf> {
    let pattern_path = cwd.join(pattern);
    let pattern_path_str = pattern_path.to_str().unwrap();

    glob(pattern_path_str).unwrap().flatten().collect()
}

// steal code: https://stackoverflow.com/a/64148190
pub fn iso8601(st: std::time::SystemTime) -> String {
    let dt: DateTime<Local> = st.clone().into();
    let dtl = dt.naive_local();
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
    let since_the_epoch = s.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_the_epoch
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
pub fn strip_pwd(pwd: &PathBuf, p: &PathBuf) -> PathBuf {
    p.strip_prefix(pwd).unwrap().parent().unwrap().to_path_buf()
}
