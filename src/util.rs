use std::path::PathBuf;
use glob::glob;

pub fn load_files(cwd: &PathBuf, pattern: &str) -> Vec<PathBuf> {
    let pattern_path = cwd.join(pattern);
    let pattern_path_str = pattern_path.to_str().unwrap();

    glob(pattern_path_str).unwrap().flatten().collect()
}
