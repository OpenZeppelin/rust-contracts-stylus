use std::fmt::Display;

use crate::{get_l2_gas_used, ArbTxReceipt};

const SEPARATOR: &str = "::";

#[derive(Debug)]
pub struct Report {
    contract: String,
    fns: Vec<(String, u128)>,
}

impl Report {
    pub fn new(contract: &str) -> Self {
        Report { contract: contract.to_owned(), fns: vec![] }
    }

    pub fn add(mut self, receipt: (&str, ArbTxReceipt)) -> eyre::Result<Self> {
        let gas = get_l2_gas_used(&receipt.1)?;
        self.fns.push((receipt.0.to_owned(), gas));
        Ok(self)
    }

    fn get_longest_signature(&self) -> usize {
        let prefix_len = self.contract.len() + SEPARATOR.len();
        self.fns
            .iter()
            .map(|(sig, _)| prefix_len + sig.len())
            .max()
            .unwrap_or_default()
    }
}

#[derive(Debug, Default)]
pub struct Reports(Vec<Report>);

impl Reports {
    pub fn merge_with(mut self, report: Report) -> Self {
        self.0.push(report);
        self
    }
}

impl Display for Reports {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reports = &self.0;
        let width = reports
            .iter()
            .map(Report::get_longest_signature)
            .max()
            .unwrap_or_default();

        for report in reports {
            let prefix = format!("{}{SEPARATOR}", report.contract);

            for (sig, gas) in &report.fns {
                let signature = format!("{prefix}{sig}");
                writeln!(f, "{signature:<width$} {gas:>10}")?;
            }
        }

        Ok(())
    }
}
