#![allow(unused_imports)]
use std::{env, fs::remove_dir_all, path::PathBuf};

#[test]
fn cli_tests() {
    let cwd = env::current_dir().unwrap();
    let new_esker_path = cwd.join("tests/example_site/_esker");

    if new_esker_path.is_dir() {
        remove_dir_all(new_esker_path.clone()).unwrap();
    }

    assert_eq!(new_esker_path.is_dir(), false);

    // -- Test esker new output tree ------

    trycmd::TestCases::new().case("tests/cmd/new.md");
    assert_eq!(new_esker_path.join("public").is_dir(), true);
    assert_eq!(new_esker_path.join("templates").is_dir(), true);

    let expected_templates = vec![
        "base.html",
        "feed.rss",
        "list.html",
        "single.html",
        "tags.html",
    ];
    let expected_public_files = vec![
        "css/main.css",
    ];

    for f in expected_templates {
        let p = new_esker_path.join("templates").join(f).is_file();
        assert_eq!(p, true)
    }

    for f in expected_public_files {
        let p = new_esker_path.join("public").join(f).is_file();
        assert_eq!(p, true)
    }

    trycmd::TestCases::new().case("tests/cmd/new_when_dir_exists.md");

    // -- Test build output ------

    trycmd::TestCases::new().case("tests/cmd/build.md");
    assert_eq!(new_esker_path.join("_site/public").is_dir(), true);
    assert_eq!(new_esker_path.join("_site/tags").is_dir(), true);
    assert_eq!(new_esker_path.join("_site/feed.rss").is_file(), true);
}
