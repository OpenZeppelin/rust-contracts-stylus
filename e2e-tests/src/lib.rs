use tokio::join;

pub mod abi;
mod erc20;
mod erc721;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn master_test() {
    join!(
        erc20::mint(),
        erc721::mint(),
        erc721::error_when_reusing_token_id(),
        erc721::transfer(),
        erc721::error_when_transfer_nonexistent_token(),
        erc721::approve_token_transfer(),
        erc721::error_when_transfer_unapproved_token(),
    );
}
