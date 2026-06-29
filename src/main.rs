use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod config;
mod crypto;
mod protocol;
mod server;
mod client;
mod ui;

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
        Commands::Serve { salt, port, mode } => {
            println!("🚀 Starting Globy Server");
            println!("📡 Port: {}", port);
            println!("🔐 Mode: {}", mode);

            let config = Config::load_or_default(&cli.config)?;
            let salt = salt.unwrap_or_else(|| config.server.salt_hash.clone());

            println!("🔑 Salt: {}", salt);

            server::run(config, port, &mode).await?;
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
