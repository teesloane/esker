use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Build your site
    Build {
        /// Directory of your site
        #[arg(short, long, value_name = "DIR")]
        dir: Option<PathBuf>
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Build { dir }) => {
            if let Some(dir_path) = dir {
                println!("Dir_path is {}", dir_path.display());
            } else {
                println!("Not printing testing lists...");
            }
        }
        None => {}
    }
}
