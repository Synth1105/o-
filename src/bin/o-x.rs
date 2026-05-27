use clap::Parser;
use o_::{report, x};
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
    x::process(&package, &version, &args.args)
}
