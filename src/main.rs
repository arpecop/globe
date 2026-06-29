use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod config;
mod crypto;
mod protocol;
mod server;
mod client;
mod ui;
mod handshake;
mod ssh_key;
mod pm;

use config::Config;

#[derive(Parser)]
#[command(name = "Globy")]
#[command(about = "P2P ephemeral chat network", long_about = None)]
#[command(version)]
#[command(author)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(global = true, short, long)]
    config: Option<PathBuf>,

    #[arg(global = true, short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run server mode
    Serve {
        /// Device salt hash (unique per machine)
        #[arg(long)]
        salt: Option<String>,

        /// Server port
        #[arg(long, default_value = "3000")]
        port: u16,

        /// Server mode: tui, api, or both
        #[arg(long, default_value = "both")]
        mode: String,
    },
    /// Run CLI client
    Cli {
        /// Server address to connect to
        #[arg(long, default_value = "localhost:3000")]
        connect: String,

        /// Nickname
        #[arg(long)]
        nickname: Option<String>,
    },
    /// Show version and info
    Version,
    /// Show server info
    Info,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    match cli.command {
        Commands::Serve { salt: _, port, mode: _ } => {
            println!("🚀 Starting Globy Server");
            println!("📡 Port: {}", port);

            let config = Config::load_or_default(&cli.config)?;

            // Get or prompt for nickname
            let nickname = std::env::var("GLOBY_NICKNAME")
                .unwrap_or_else(|_| {
                    println!("👤 Enter your nickname: ");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).ok();
                    input.trim().to_string()
                });

            println!("👤 Nickname: {}", nickname);

            server::run(config, port, nickname).await?;
        }
        Commands::Cli { connect, nickname } => {
            println!("🌐 Connecting to {}", connect);
            client::run(&connect, nickname).await?;
        }
        Commands::Version => {
            println!("Globy v{}", env!("CARGO_PKG_VERSION"));
            println!("P2P ephemeral chat network");
        }
        Commands::Info => {
            let device_id = uuid::Uuid::new_v4().to_string();
            println!("Globy Information");
            println!("─────────────────");
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            println!("Device ID: {}", device_id);
            println!("Home: ~/.globy/");
        }
    }

    Ok(())
}
