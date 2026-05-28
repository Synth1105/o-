pub mod app_error;
pub mod args;
pub mod binengine;
pub mod conf;
pub mod lock;
pub mod pm;
pub mod report;
pub mod toolchain;
pub mod x;

pub use app_error::AppError;
use args::Commands;
use home::home_dir;
pub use o_core::engine;
use o_core::engine::JSEngine;
pub use o_core::error;
use std::fs;
use std::path::PathBuf;

use crate::args::ToolChainCommand;
use crate::binengine::BinEngine;
use crate::pm::global_install;
use crate::report::Report;

pub fn process(args: Commands, toolchain: Option<&str>) -> Result<(), AppError> {
    match args {
        Commands::Run { path } => {
            let toolchain_name = toolchain.unwrap_or("").trim().to_string();
            let selected_toolchain = select_toolchain(&toolchain_name)?;
            run(&path, &toolchain_name, selected_toolchain)
        }
        Commands::Toolchain { command } => {
            let report = run_toolchain(command)?;
            report::print(&report);
            Ok(())
        }
        Commands::Install { global, package } => {
            if global {
                let report =
                    global_install(package.as_deref()).map_err(AppError::PackageManager)?;
                report::print(&report);
                Ok(())
            } else {
                let report = pm::install().map_err(AppError::PackageManager)?;
                report::print(&report);
                Ok(())
            }
        }
        Commands::Uninstall { name } => {
            let report = pm::uninstall(&name).map_err(AppError::PackageManager)?;
            report::print(&report);
            Ok(())
        }
    }
}

fn run_toolchain(command: ToolChainCommand) -> Result<Report, AppError> {
    match command {
        ToolChainCommand::Add { user, repo } => {
            let mut installed =
                toolchain::install("github.com", &user, &repo).map_err(|source| {
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

fn select_toolchain(toolchain: &str) -> Result<Box<dyn JSEngine>, AppError> {
    let toolchain = toolchain.trim();
    if toolchain.is_empty() {
        return Err(AppError::MissingToolchainSelection);
    }

    let path = resolve_toolchain_path(toolchain)?;
    if !path.is_file() {
        return Err(AppError::ToolchainNotInstalled {
            toolchain: toolchain.to_string(),
            path,
        });
    }

    Ok(Box::new(BinEngine::new(
        path.to_string_lossy().into_owned(),
    )))
}

fn run(path: &str, toolchain_name: &str, toolchain: Box<dyn JSEngine>) -> Result<(), AppError> {
    let file = match fs::read_to_string(path) {
        Ok(file) => file,
        Err(source) => {
            return Err(AppError::ReadScript {
                path: PathBuf::from(path),
                source,
            });
        }
    };

    if let Err(err) = toolchain.run(&file, path) {
        return Err(AppError::ToolchainExecution {
            toolchain: toolchain_name.to_string(),
            message: err.to_string(),
        });
    }

    Ok(())
}

fn resolve_toolchain_path(name: &str) -> Result<PathBuf, AppError> {
    let mut toolchain: PathBuf = home_dir().ok_or(AppError::HomeDirUnavailable)?;
    toolchain.push(".config");
    toolchain.push("o-");
    toolchain.push("toolchains");
    toolchain.push("bin");
    toolchain.push(name);
    Ok(toolchain)
}
