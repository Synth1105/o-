use o_core::engine::JSEngine;
use o_core::error::JSResult;
use std::{env, io, process};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    match (
        args.next().as_deref(),
        args.next(),
        args.next(),
        args.next(),
    ) {
        (Some("-c"), Some(filename), Some(source), None) => {
            let engine = o_toolchain_spidermonkey::SpiderMonkey::new();
            match engine.run(&source, &filename)? {
                JSResult::String(output) => {
                    println!("{output}");
                    Ok(())
                }
            }
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: o-toolchain-spidermonkey -c <filename> <source>",
        )
        .into()),
    }
}
