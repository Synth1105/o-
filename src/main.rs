use clap::Parser;
use o_::{AppError, args::Args, conf, report};
use std::{fs, process};

fn main() {
    if let Err(error) = _main() {
        report::print_error(&error.report());
        process::exit(1);
    }
}

fn _main() -> Result<(), AppError> {
    let args = Args::parse();
    let toolchain = match &args.command {
        o_::args::Commands::Run { .. } => {
            let home = home::home_dir().ok_or(AppError::HomeDirUnavailable)?;
            let mut config = home;
            config.push(".config");
            config.push("o-");
            config.push("config.toml");
            let config_file =
                fs::read_to_string(&config).map_err(|source| AppError::ReadConfig {
                    path: config.clone(),
                    source,
                })?;
            Some(conf::parse_config(&config_file)?)
        }
        _ => None,
    };
    o_::process(args.command, toolchain.as_deref())
}
