#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod config;
pub mod errors;
pub mod frontmatter;
pub mod link;
pub mod md_file;
pub mod new_site;
pub mod parser;
pub mod site;
pub mod templates;
pub mod util;

use axum::{http::StatusCode, routing::get_service, Router};
use clap::{Parser, Subcommand};
use colored::*;
use hotwatch::Hotwatch;
use parser::syntax_highlight::dump_syntax_binary;
use site::Site;
use std::{net::SocketAddr, path::PathBuf, thread, time::Duration};
use tower_http::services::ServeDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help(true))]
pub struct Cli {
    /// Directory of where you want to run esker
    #[arg(short, long, value_name = "DIR", global = true)]
    dir: Option<PathBuf>,

    /// Whether or not to print errors
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Compile your site to /_esker/_site
    Build,
    #[command(hide = true)]
    DumpSyntax,
    /// Create a new _esker site in your directory.
    New,
    /// Run a local server and reload your site on change.
    Watch {
        #[clap(short, long, default_value_t = 8080)]
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Watch { port }) => watch(Commands::Watch { port: *port }, cli).await,
        Some(Commands::New) => new_site::init(cli.dir),
        Some(Commands::DumpSyntax) => dump_syntax_binary(),
        Some(Commands::Build) => {
            let mut site = Site::new(Commands::Build, cli);
            site.build();
            println!("{}: site built!", " Success".green().on_black());

        }
        None => {}
    }
}

async fn watch(cmd: Commands, cli: Cli) {
    let mut site = Site::new(cmd.clone(), cli);
    site.build();
    let output_directory = site.dir_esker_site.clone();

    tokio::task::spawn_blocking(move || {
        let mut hotwatch =
            Hotwatch::new_with_custom_delay(Duration::from_millis(1000)).expect("hotwatch failed");

        hotwatch
            .watch(".", move |event| {
                site.handle_watch_event(event);
            })
            .expect("failed");

        loop {
            thread::sleep(Duration::from_secs(1));
        }
    });

    let app = Router::new().fallback_service(
        get_service(ServeDir::new(output_directory).append_index_html_on_directories(true))
            .handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            }),
    );

    if let Commands::Watch { port } = cmd {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        println!("listening on {}", addr);

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}
