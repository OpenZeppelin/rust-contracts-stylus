use alloy::{
    primitives::Address,
    rpc::types::{
        serde_helpers::WithOtherFields, AnyReceiptEnvelope, Log,
        TransactionReceipt,
    },
};
use alloy_primitives::U128;
use e2e::{Account, ReceiptExt};
use koba::config::{Deploy, Generate, PrivateKey};
use serde::Deserialize;

pub mod access_control;
pub mod erc20;
pub mod erc721;
pub mod merkle_proofs;
pub mod report;

const RPC_URL: &str = "http://localhost:8547";

#[derive(Debug, Deserialize)]
struct ArbOtherFields {
    #[serde(rename = "gasUsedForL1")]
    gas_used_for_l1: U128,
    #[allow(dead_code)]
    #[serde(rename = "l1BlockNumber")]
    l1_block_number: String,
}

type ArbTxReceipt =
    WithOtherFields<TransactionReceipt<AnyReceiptEnvelope<Log>>>;

fn get_l2_gas_used(receipt: &ArbTxReceipt) -> eyre::Result<u128> {
    let l2_gas = receipt.gas_used;
    let arb_fields: ArbOtherFields = receipt.other.deserialize_as()?;
    let l1_gas = arb_fields.gas_used_for_l1.to::<u128>();
    Ok(l2_gas - l1_gas)
}

async fn deploy(
    account: &Account,
    contract_name: &str,
    args: Option<String>,
) -> Address {
    let manifest_dir =
        std::env::current_dir().expect("should get current dir from env");

    let wasm_path = manifest_dir
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join(format!("{}_example.wasm", contract_name.replace('-', "_")));
    let sol_path = args.as_ref().map(|_| {
        manifest_dir
            .join("examples")
            .join(format!("{}", contract_name))
            .join("src")
            .join("constructor.sol")
    });

    let pk = account.pk();
    let config = Deploy {
        generate_config: Generate {
            wasm: wasm_path.clone(),
            sol: sol_path,
            args,
            legacy: false,
        },
        auth: PrivateKey {
            private_key_path: None,
            private_key: Some(pk),
            keystore_path: None,
            keystore_password_path: None,
        },
        endpoint: RPC_URL.to_owned(),
        deploy_only: false,
        quiet: true,
    };

    koba::deploy(&config)
        .await
        .expect("should deploy contract")
        .address()
        .expect("should return contract address")
}
