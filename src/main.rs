use clap::Parser;
use o_::{AppError, args::Args, conf, report};
use std::{fs, process};

fn main() {
    if let Err(error) = try_main() {
        report::print_error(&error.report());
        process::exit(1);
    }
}

fn try_main() -> Result<(), AppError> {
    let home = home::home_dir().ok_or(AppError::HomeDirUnavailable)?;
    let mut config = home;
    config.push(".config");
    config.push("o-");
    config.push("config.toml");
    let args = Args::parse();
    let config_file = fs::read_to_string(&config).map_err(|source| AppError::ReadConfig {
        path: config.clone(),
        source,
    })?;
    let toolchain = conf::parse_config(&config_file)?;
    o_::process(args.command, &toolchain)
}
