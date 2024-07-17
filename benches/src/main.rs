use benches::{access_control, erc20, erc721, merkle_proofs};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _ = tokio::join!(
        access_control::bench(),
        erc20::bench(),
        erc721::bench(),
        merkle_proofs::bench()
    );

    Ok(())
}
