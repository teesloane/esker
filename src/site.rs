use std::{path::PathBuf, env};

#[derive(Debug)]
pub struct Site {
    pub dir: PathBuf
}

impl Site {
    pub fn new(dir: Option<PathBuf>) -> Site {
        let cwd: PathBuf;
        if let Some(dir) = dir {
            cwd = dir;
        } else {
            cwd = env::current_dir().unwrap();
        }

        Site {
            dir: cwd

        }
    }
}
