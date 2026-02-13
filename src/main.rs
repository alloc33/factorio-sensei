mod cli;
mod repl;

use std::sync::Arc;

use clap::Parser;
use factorio_rcon::RconClient;
use factorio_sensei::agent;
use tokio::sync::Mutex;

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    let rt = tokio::runtime::Runtime::new()?;

    eprintln!("{DIM}Connecting to Factorio RCON at {}...{RESET}", cli.addr);
    let rcon = rt.block_on(async {
        let client = RconClient::connect(&cli.addr, &cli.password).await?;
        Ok::<_, anyhow::Error>(Arc::new(Mutex::new(client)))
    })?;

    let model_name = cli.model.as_deref().unwrap_or(agent::DEFAULT_MODEL);
    eprintln!("{DIM}Connected! Model: {model_name}. Type /help for commands.{RESET}\n");

    let coach = agent::build_coach(&rcon, cli.model.as_deref());

    repl::run(&rt, &coach)
}
