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

    let wiki_dir = std::path::Path::new("data/wiki");
    let wiki_articles = if wiki_dir.exists() {
        match factorio_sensei::knowledge::load_wiki_articles(wiki_dir) {
            Ok(articles) => {
                eprintln!(
                    "{DIM}Loaded {} knowledge article(s).{RESET}",
                    articles.len()
                );
                articles
            }
            Err(e) => {
                eprintln!("{DIM}Warning: could not load wiki ({e}), continuing without knowledge base.{RESET}");
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    let model_name = cli.model.as_deref().unwrap_or(agent::DEFAULT_MODEL);
    eprintln!("{DIM}Connected! Model: {model_name}. Type /help for commands.{RESET}\n");

    let coach = agent::build_coach(&rcon, cli.model.as_deref(), &wiki_articles);

    if cli.bridge {
        let bridge_rcon = rcon.clone();
        let bridge_coach = coach.clone();
        rt.spawn(async move {
            factorio_sensei::bridge::run(
                bridge_rcon,
                bridge_coach,
                std::time::Duration::from_secs(2),
            )
            .await;
        });
        eprintln!("{DIM}In-game /coach bridge enabled.{RESET}");
    }

    repl::run(&rt, &coach)
}
