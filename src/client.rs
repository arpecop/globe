use anyhow::Result;

pub async fn run(connect: &str, nickname: Option<String>) -> Result<()> {
    println!("🌐 Connecting to {}", connect);

    let nickname = nickname.unwrap_or_else(|| "User".to_string());
    println!("👤 Nickname: {}", nickname);

    // TODO: Implement TUI client
    println!("⏳ TUI client not yet implemented");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
