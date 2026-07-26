#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    swayve::cli::run_cli().await?;
    Ok(())
}
