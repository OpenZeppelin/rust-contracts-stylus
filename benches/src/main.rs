use benches::{access_control, erc20, erc721, merkle_proofs};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _ = tokio::join!(
        erc20::bench(),
        erc721::bench(),
        access_control::bench(),
        merkle_proofs::bench()
    );

    Ok(())
}
