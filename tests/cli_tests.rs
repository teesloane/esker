#![allow(unused_imports)]
use std::{env, fs::remove_dir_all, path::PathBuf};

#[test]
fn cli_tests() {
    let cwd = env::current_dir().unwrap();
    let new_esker_path = cwd.join("_esker");
    remove_dir_all(new_esker_path).unwrap();

    trycmd::TestCases::new().case("tests/cmd/new.md");
    trycmd::TestCases::new().case("tests/cmd/new_when_dir_exists.md");
    trycmd::TestCases::new().case("tests/cmd/build.md");
}
