use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Commands {
    Run {
        path: String,
    },
    Toolchain {
        #[command(subcommand)]
        command: ToolChainCommand,
    },
    Install {
        #[arg(short, long)]
        global: bool,
        package: Option<String>,
    },
    Uninstall {
        name: String,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToolChainCommand {
    Add { user: String, repo: String },
    Remove { toolchain: String },
}
