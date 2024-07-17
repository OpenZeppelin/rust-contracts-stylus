use std::fmt::Display;

#[derive(Debug)]
pub struct Report {
    contract: String,
    fns: Vec<(String, u128)>,
}

impl Report {
    pub fn new(contract: &str) -> Self {
        Report { contract: contract.to_owned(), fns: vec![] }
    }

    pub fn add(&mut self, signature: &str, gas: u128) {
        self.fns.push((signature.to_owned(), gas));
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
        let separator = "::";
        let mut max_width = 0;
        for report in &self.0 {
            let prefix_len = report.contract.len() + separator.len();
            let width = report
                .fns
                .iter()
                .map(|(sig, _)| prefix_len + sig.len())
                .max()
                .unwrap_or(0);
            max_width = max_width.max(width);
        }

        for report in &self.0 {
            let prefix = format!("{}{separator}", report.contract);

            for (sig, gas) in &report.fns {
                let signature = format!("{prefix}{sig}");
                writeln!(f, "{signature:<max_width$} {gas:>}")?;
            }
        }

        Ok(())
    }
}
