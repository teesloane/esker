#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub mod errors;
pub mod frontmatter;
pub mod link;
pub mod md_file;
pub mod site;
pub mod util;
pub mod templates;
pub mod config;
pub mod parser;
pub mod new_site;

use clap::{Parser, Subcommand};
use site::Site;
use std::path::PathBuf;

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
    Build,
    DumpSyntax,
    New
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::New) => {
            let s = Site::init(cli.dir);
        }
        Some(Commands::Build) => {
            let s = Site::build(cli.dir);
        }

        Some(Commands::DumpSyntax) => {
            parser::dump_syntax_binary();

        }
        None => {}
    }
}
