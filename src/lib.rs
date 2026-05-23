pub mod args;
pub mod app_error;
pub mod conf;
pub mod binengine;
pub mod report;
pub mod toolchain;

use std::fs;
use std::path::PathBuf;
use args::Commands;
use home::home_dir;
pub use app_error::AppError;
pub use o_core::engine;
use o_core::engine::JSEngine;
pub use o_core::error;
use o_toolchain_javascriptcore::JavaScriptCore;
use o_toolchain_spidermonkey::SpiderMonkey;
use o_toolchain_v8::V8Engine;

use crate::args::ToolChainCommand;
use crate::binengine::BinEngine;
use crate::report::Report;
use crate::toolchain::install;

pub fn process(args: Commands, toolchain: &str) -> Result<(), AppError> {
    match args {
        Commands::Run { path } => {
            let selected_toolchain = select_toolchain(toolchain)?;
            run(&path, selected_toolchain);
            Ok(())
        }
        Commands::Toolchain { command } => {
            let report = run_toolchain(command)?;
            report::print(&report);
            Ok(())
        }
    }
}

fn run_toolchain(command: ToolChainCommand) -> Result<Report, AppError> {
    match command {
        ToolChainCommand::Add { user, repo } => {
            let mut installed = install("github.com", &user, &repo).map_err(|source| {
                AppError::InstallToolchain {
                    user: user.clone(),
                    repo: repo.clone(),
                    source: source.to_string(),
                }
            })?;
            installed.push("bin");
            installed.push(&repo);
            let mut target = home_dir().ok_or(AppError::HomeDirUnavailable)?;
            target.push(".config");
            target.push("o-");
            target.push("toolchains");
            target.push("bin");
            target.push(&repo);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|source| AppError::CreateDir {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            fs::rename(&installed, &target).map_err(|source| AppError::MoveToolchain {
                from: installed.clone(),
                to: target.clone(),
                source,
            })?;

            Ok(Report::new(format!("installed toolchain `{repo}`"))
                .detail(format!("source: {user}/{repo}"))
                .detail(format!("binary: {}", target.display())))
        }
        ToolChainCommand::Remove { toolchain } => {
            let mut target = home::home_dir().ok_or(AppError::HomeDirUnavailable)?;
            target.push(".config");
            target.push("o-");
            target.push("toolchains");
            target.push("bin");
            target.push(&toolchain);
            fs::remove_file(&target).map_err(|source| AppError::RemoveToolchain {
                path: target.clone(),
                source,
            })?;

            Ok(Report::new(format!("removed toolchain `{toolchain}`"))
                .detail(format!("path: {}", target.display())))
        }
    }
}

fn resolve_toolchain(name: &str) -> Result<String, AppError> {
    let mut toolchain: PathBuf = home::home_dir().ok_or(AppError::HomeDirUnavailable)?;
    toolchain.push(".config");
    toolchain.push("o-");
    toolchain.push("toolchains");
    toolchain.push(name);
    Ok(toolchain.to_string_lossy().into_owned())
}

fn select_toolchain(toolchain: &str) -> Result<Box<dyn JSEngine>, AppError> {
    let engine: Box<dyn JSEngine> = match toolchain.trim() {
        "javascriptcore" | "jsc" => Box::new(JavaScriptCore::new()),
        "v8" => Box::new(V8Engine::new()),
        "spidermonkey" | "" => Box::new(SpiderMonkey::new()),
        other => Box::new(BinEngine::new(resolve_toolchain(other)?)),
    };
    Ok(engine)
}

fn run(path: &str, toolchain: Box<dyn JSEngine>) {
    let file = match fs::read_to_string(path) {
        Ok(file) => file,
        Err(source) => {
            let error = AppError::ReadScript {
                path: PathBuf::from(path),
                source,
            };
            report::print_error(&error.report());
            return;
        }
    };

    if let Err(err) = toolchain.run(&file, path) {
        eprintln!("{err}");
    }
}
