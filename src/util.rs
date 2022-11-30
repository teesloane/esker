use std::time::{SystemTime, UNIX_EPOCH};
use std::{path::PathBuf};
use glob::glob;
use chrono::prelude::{DateTime, Local};
use std::path::Path;
use std::fs::File;
use std::io::{self, BufRead};

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
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn get_time_in_ms(s: SystemTime) -> std::time::Duration  {
    let since_the_epoch = s
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch
}
