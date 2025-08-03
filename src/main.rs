use msr_downloader::{Downloader, MonsterSirenClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let version = option_env!("CARGO_PKG_VERSION");

    println!("Monster Siren Downloader v{}", version.unwrap_or("dev"));
    println!("Starting Monster Siren Records music library download...");

    let client = MonsterSirenClient::new(version)?;
    let downloader = Downloader::new(client);

    downloader.download_all_tracks().await?;

    println!("All downloads completed!");
    Ok(())
}
