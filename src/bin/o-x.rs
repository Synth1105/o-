use clap::Parser;
use o_::{report, x};
use std::io::{self, Write};
use std::process;

fn main() {
    if let Err(error) = run() {
        report::print_error(&error.report());
        process::exit(1);
    }
}

fn run() -> Result<(), o_::pm::PmError> {
    let args = x::Args::parse();
    let (package, version) = x::parse_package(&args.package)?;
    let stdout = x::process(&package, &version, &args.args)?;
    if !stdout.is_empty() {
        io::stdout()
            .write_all(stdout.as_bytes())
            .map_err(|source| o_::pm::PmError::WriteProcessOutput { source })?;
    }
    Ok(())
}
