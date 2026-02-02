use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "agent")]
#[command(about = "SuperAgent CLI", long_about = None)]
pub struct Commands {
    #[command(subcommand)]
    pub command: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    Run { #[arg(short, long)] goal: String },
    Chat,
    Graph,
    Logs,
    Tui,
    Exit,
    Models { #[command(subcommand)] cmd: ModelCmd },
}

#[derive(Subcommand, Debug)]
pub enum ModelCmd {
    List,
    Import { #[arg()] path: String },
    Remove { #[arg()] name: String },
    Serve { #[arg()] action: String, #[arg()] model: Option<String> },
    Install { #[arg()] tool: Option<String> },
}
