use anyhow::Result;
use clap::Parser;
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
mod api;

use config::Config;
use ssh_key::SshIdentity;

#[derive(Parser)]
#[command(name = "globy")]
#[command(about = "P2P ephemeral chat network")]
#[command(version)]
struct Cli {
    /// Run as relay server (host mode)
    #[arg(long)]
    host: bool,

    /// Connect to relay server (default: 130.204.65.82:3000)
    #[arg(long)]
    relay: Option<String>,

    /// Show your SSH public key and peer ID
    #[arg(long)]
    show_key: bool,

    /// Server/client port (default: 3000)
    #[arg(long, default_value = "3000")]
    port: u16,

    /// Your nickname
    #[arg(long)]
    nickname: Option<String>,

    #[arg(global = true, short, long)]
    config: Option<PathBuf>,

    #[arg(global = true, short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    // Show SSH key and exit
    if cli.show_key {
        match SshIdentity::new() {
            Ok(identity) => {
                let peer_id = identity.get_peer_hash()?;
                let pub_key = identity.get_public_key()?;
                println!("🔑 SSH Public Key:");
                println!("{}", pub_key);
                println!("🆔 Your Peer ID: {}", peer_id);
            }
            Err(_) => {
                println!("❌ No SSH key found at ~/.ssh/id_ed25519.pub");
                println!("Generate one with: ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519");
            }
        }
        return Ok(());
    }

    let config = Config::load_or_default(&cli.config)?;
    let nickname = cli.nickname.unwrap_or_else(|| {
        println!("👤 Enter your nickname: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        input.trim().to_string()
    });

    if cli.host {
        // Run as relay server
        println!("🚀 Starting Globy Relay");
        println!("📡 Port: {}", cli.port);
        server::run(config, cli.port, nickname).await?;
    } else {
        // Connect to relay as client
        let relay = cli.relay.unwrap_or_else(|| "130.204.65.82:3000".to_string());
        println!("🌐 Connecting to relay: {}", relay);
        client::run(&relay, Some(nickname)).await?;
    }

    Ok(())
}
