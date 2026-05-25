const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const RED: &str = "\x1b[31m";

pub struct Report {
    summary: String,
    details: Vec<String>,
}

impl Report {
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            details: Vec::new(),
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }
}

pub fn print(report: &Report) {
    eprintln!("{CYAN}o-{RESET} {}", report.summary);
    for detail in &report.details {
        eprintln!("{DIM}- {detail}{RESET}");
    }
}

pub fn print_error(report: &Report) {
    eprintln!("{RED}error{RESET} {}", report.summary);
    for detail in &report.details {
        eprintln!("{DIM}- {detail}{RESET}");
    }
}
