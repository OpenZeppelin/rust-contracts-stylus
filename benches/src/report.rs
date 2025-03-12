use std::{collections::HashMap, fmt::Display, future::Future};

use alloy::network::AnyTransactionReceipt;

use crate::{ArbOtherFields, Opt};

const SEPARATOR: &str = "::";

#[derive(Debug)]
pub struct FunctionReport {
    sig: String,
    gas: u128,
}

impl FunctionReport {
    pub(crate) fn new(
        receipt: (&str, AnyTransactionReceipt),
    ) -> eyre::Result<Self> {
        Ok(FunctionReport {
            sig: receipt.0.to_owned(),
            gas: get_l2_gas_used(&receipt.1)?,
        })
    }
}

#[derive(Debug)]
pub struct ContractReport {
    contract: String,
    functions: Vec<FunctionReport>,
    functions_cached: Vec<FunctionReport>,
    functions_wasm_opt_cached: Vec<FunctionReport>,
}

impl ContractReport {
    pub async fn generate<
        F: Future<Output = eyre::Result<Vec<FunctionReport>>>,
    >(
        name: &str,
        run: impl Fn(Opt) -> F,
    ) -> eyre::Result<ContractReport> {
        let reports = run(Opt::None).await?;
        let report = reports
            .into_iter()
            .try_fold(ContractReport::new(name), ContractReport::add)?;

        let cached_reports = run(Opt::Cache).await?;
        let report = cached_reports
            .into_iter()
            .try_fold(report, ContractReport::add_cached)?;

        let wasm_opt_cached_reports = run(Opt::CacheWasmOpt).await?;
        let report = wasm_opt_cached_reports
            .into_iter()
            .try_fold(report, ContractReport::add_wasm_opt_cached)?;

        Ok(report)
    }

    pub fn new(contract: &str) -> Self {
        ContractReport {
            contract: contract.to_owned(),
            functions: vec![],
            functions_cached: vec![],
            functions_wasm_opt_cached: vec![],
        }
    }

    pub fn add(mut self, fn_report: FunctionReport) -> eyre::Result<Self> {
        self.functions.push(fn_report);
        Ok(self)
    }

    pub fn add_cached(
        mut self,
        fn_report: FunctionReport,
    ) -> eyre::Result<Self> {
        self.functions_cached.push(fn_report);
        Ok(self)
    }

    pub fn add_wasm_opt_cached(
        mut self,
        fn_report: FunctionReport,
    ) -> eyre::Result<Self> {
        self.functions_wasm_opt_cached.push(fn_report);
        Ok(self)
    }

    fn signature_max_len(&self) -> usize {
        let prefix_len = self.contract.len() + SEPARATOR.len();
        self.functions
            .iter()
            .map(|FunctionReport { sig: name, .. }| prefix_len + name.len())
            .max()
            .unwrap_or_default()
    }

    fn gas_max_len(&self) -> usize {
        self.functions
            .iter()
            .map(|FunctionReport { gas, .. }| gas.to_string().len())
            .max()
            .unwrap_or_default()
    }

    fn gas_cached_max_len(&self) -> usize {
        self.functions_cached
            .iter()
            .map(|FunctionReport { gas, .. }| gas.to_string().len())
            .max()
            .unwrap_or_default()
    }

    fn gas_wasm_opt_cached_max_len(&self) -> usize {
        self.functions_wasm_opt_cached
            .iter()
            .map(|FunctionReport { gas, .. }| gas.to_string().len())
            .max()
            .unwrap_or_default()
    }
}

#[derive(Debug, Default)]
pub struct BenchmarkReport(Vec<ContractReport>);

impl BenchmarkReport {
    pub fn merge_with(mut self, report: ContractReport) -> Self {
        self.0.push(report);
        self
    }

    pub fn column_width(
        &self,
        column_value: impl FnMut(&ContractReport) -> usize,
        header: &str,
    ) -> usize {
        self.0
            .iter()
            .map(column_value)
            .chain(std::iter::once(header.len()))
            .max()
            .unwrap_or_default()
    }
}

impl Display for BenchmarkReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const HEADER_SIG: &str = "Contract::function";
        const HEADER_WASM_OPT_GAS_CACHED: &str = "WASM Opt & Cached";
        const HEADER_GAS_CACHED: &str = "Cached";
        const HEADER_GAS: &str = "Not Cached";

        // Calculating the width of table columns.
        let width1 =
            self.column_width(ContractReport::signature_max_len, HEADER_SIG);
        let width2 = self.column_width(
            ContractReport::gas_wasm_opt_cached_max_len,
            HEADER_WASM_OPT_GAS_CACHED,
        );
        let width3 = self.column_width(
            ContractReport::gas_cached_max_len,
            HEADER_GAS_CACHED,
        );
        let width4 = self.column_width(ContractReport::gas_max_len, HEADER_GAS);

        // Print headers for the table columns.
        writeln!(
            f,
            "| {HEADER_SIG:<width1$} | {HEADER_WASM_OPT_GAS_CACHED:>width2$} | {HEADER_GAS_CACHED:>width3$} | {HEADER_GAS:>width4$} |"
        )?;
        writeln!(
            f,
            "| {:->width1$} | {:->width2$} | {:->width3$} | {:->width4$} |",
            "", "", "", "",
        )?;

        // Merging a non-cached gas report with a cached one.
        for report in &self.0 {
            let prefix = format!("{}{SEPARATOR}", report.contract);

            let wasm_opt_cached: HashMap<_, _> = report
                .functions_wasm_opt_cached
                .iter()
                .map(|func| (&*func.sig, func.gas))
                .collect();

            let gas_cached: HashMap<_, _> = report
                .functions_cached
                .iter()
                .map(|func| (&*func.sig, func.gas))
                .collect();

            for report in &report.functions {
                let sig = &*report.sig;
                let gas_wasm_opt_cached = wasm_opt_cached
                    .get(sig)
                    .map(u128::to_string)
                    .unwrap_or("—".to_string());
                let gas_cached = gas_cached
                    .get(sig)
                    .map(u128::to_string)
                    .unwrap_or("—".to_string());
                let gas = &report.gas;

                let full_sig = format!("{prefix}{sig}");
                writeln!(
                    f,
                    "| {full_sig:<width1$} | {gas_wasm_opt_cached:>width2$} | {gas_cached:>width3$} | {gas:>width4$} |"
                )?;
            }
        }

        Ok(())
    }
}

const BASE_GAS_FEE: u128 = 21_000;

fn get_l2_gas_used(receipt: &AnyTransactionReceipt) -> eyre::Result<u128> {
    let l2_gas = receipt.gas_used;
    let arb_fields: ArbOtherFields = receipt.other.deserialize_as()?;
    let l1_gas = arb_fields.gas_used_for_l1.to::<u128>();
    Ok(l2_gas - l1_gas - BASE_GAS_FEE)
}
