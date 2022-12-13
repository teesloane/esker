#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]

pub mod site;
pub mod util;
pub mod md_file;
pub mod frontmatter;

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use site::Site;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {

    /// Directory of where you want to run esker
    #[arg(short, long, value_name = "DIR", global = true)]
    dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Build your site
    Build
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Build) => {
            let s = Site::new(cli.dir);
            // println!("{:#?}", s);
        }
        None => {}
    }
}
