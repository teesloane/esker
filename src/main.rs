#![allow(dead_code)]
#![allow(unused_mut)]
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

use axum::{
    http::StatusCode, routing::get_service, Router,
};
use clap::{Parser, Subcommand};
use hotwatch::Hotwatch;
use site::Site;
use std::{
    net::SocketAddr,
    path::PathBuf,
    thread,
    time::Duration,
};
use tower_http::services::ServeDir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Directory of where you want to run esker
    #[arg(short, long, value_name = "DIR", global = true)]
    dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Build your site
    Build,
    DumpSyntax,
    New,
    Watch {
        #[clap(short, long, default_value_t = 8080)]
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Watch { port }) => {
            watch(cli.dir, Commands::Watch { port: *port }).await;
        }

        Some(Commands::New) => {
            new_site::init(cli.dir);
        }
        Some(Commands::Build) => {
            let mut site = Site::new(cli.dir, Commands::Build);
            site.build()
        }

        Some(Commands::DumpSyntax) => {
            parser::syntax_highlight::dump_syntax_binary();
        }
        None => {}
    }
}

async fn watch(dir: Option<PathBuf>, cmd: Commands) {
    let mut site = Site::new(dir, cmd.clone());
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
