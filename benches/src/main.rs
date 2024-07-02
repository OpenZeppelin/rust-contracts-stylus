use benches::erc20;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    erc20::bench().await?;

    Ok(())
}
