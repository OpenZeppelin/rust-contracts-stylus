use benches::{access_control, erc20, merkle_proofs};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    erc20::bench().await?;
    access_control::bench().await?;
    merkle_proofs::bench().await?;

    Ok(())
}
