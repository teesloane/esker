#![allow(unused_imports)]
use std::{env, fs::remove_dir_all, path::PathBuf};

#[test]
fn cli_tests() {
    let cwd = env::current_dir().unwrap();
    let new_esker_path = cwd.join("_esker");

    if new_esker_path.is_dir() {
        remove_dir_all(new_esker_path.clone()).unwrap();
    }

    assert_eq!(new_esker_path.is_dir(), false);
    trycmd::TestCases::new().case("tests/cmd/new.md");
    // TODO: write tests to check that public/templates/etc exist
    assert_eq!(new_esker_path.is_dir(), true);

    trycmd::TestCases::new().case("tests/cmd/new_when_dir_exists.md");


    trycmd::TestCases::new().case("tests/cmd/build.md");
    // TODO: make sure that the structure of an _esker site is being created after build runs (_site)
    //
}
